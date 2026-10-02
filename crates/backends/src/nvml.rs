use acercontrol_core::*;
use crate::HardwareBackend;
use log::{debug, info};
use std::ffi::{c_char, c_int, c_uint, c_ulonglong, CStr, CString};
use std::sync::Mutex;

type NvmlReturn = c_int;
type NvmlDevice = *mut std::ffi::c_void;

const NVML_SUCCESS: NvmlReturn = 0;
const NVML_TEMPERATURE_GPU: c_int = 0;

#[repr(C)]
#[derive(Default, Debug)]
struct NvmlUtilization {
    gpu: c_uint,
    memory: c_uint,
}

#[repr(C)]
#[derive(Default, Debug)]
struct NvmlMemory {
    total: c_ulonglong,
    free: c_ulonglong,
    used: c_ulonglong,
}

struct NvmlBindings {
    lib: *mut std::ffi::c_void,
    init: unsafe extern "C" fn() -> NvmlReturn,
    shutdown: unsafe extern "C" fn() -> NvmlReturn,
    get_count: unsafe extern "C" fn(*mut c_uint) -> NvmlReturn,
    get_handle: unsafe extern "C" fn(c_uint, *mut NvmlDevice) -> NvmlReturn,
    get_name: unsafe extern "C" fn(NvmlDevice, *mut c_char, c_uint) -> NvmlReturn,
    get_temperature: unsafe extern "C" fn(NvmlDevice, c_int, *mut c_uint) -> NvmlReturn,
    get_utilization: unsafe extern "C" fn(NvmlDevice, *mut NvmlUtilization) -> NvmlReturn,
    get_memory: unsafe extern "C" fn(NvmlDevice, *mut NvmlMemory) -> NvmlReturn,
    get_power: unsafe extern "C" fn(NvmlDevice, *mut c_uint) -> NvmlReturn,
}

unsafe impl Send for NvmlBindings {}
unsafe impl Sync for NvmlBindings {}

impl Drop for NvmlBindings {
    fn drop(&mut self) {
        unsafe {
            let _ = (self.shutdown)();
            libc::dlclose(self.lib);
        }
    }
}

impl NvmlBindings {
    fn load() -> Option<Self> {
        let lib_names = ["libnvidia-ml.so.1", "libnvidia-ml.so"];
        for name in lib_names {
            let c_name = CString::new(name).ok()?;
            let lib = unsafe { libc::dlopen(c_name.as_ptr(), libc::RTLD_NOW) };
            if lib.is_null() {
                continue;
            }

            unsafe {
                let init = libc::dlsym(lib, b"nvmlInit_v2\0".as_ptr() as *const c_char);
                let shutdown = libc::dlsym(lib, b"nvmlShutdown\0".as_ptr() as *const c_char);
                let get_count = libc::dlsym(lib, b"nvmlDeviceGetCount_v2\0".as_ptr() as *const c_char);
                let get_handle = libc::dlsym(lib, b"nvmlDeviceGetHandleByIndex_v2\0".as_ptr() as *const c_char);
                let get_name = libc::dlsym(lib, b"nvmlDeviceGetName\0".as_ptr() as *const c_char);
                let get_temperature = libc::dlsym(lib, b"nvmlDeviceGetTemperature\0".as_ptr() as *const c_char);
                let get_utilization = libc::dlsym(lib, b"nvmlDeviceGetUtilizationRates\0".as_ptr() as *const c_char);
                let get_memory = libc::dlsym(lib, b"nvmlDeviceGetMemoryInfo\0".as_ptr() as *const c_char);
                let get_power = libc::dlsym(lib, b"nvmlDeviceGetPowerUsage\0".as_ptr() as *const c_char);

                if init.is_null() || shutdown.is_null() || get_handle.is_null() || get_temperature.is_null() {
                    libc::dlclose(lib);
                    continue;
                }

                let bindings = Self {
                    lib,
                    init: std::mem::transmute(init),
                    shutdown: std::mem::transmute(shutdown),
                    get_count: std::mem::transmute(get_count),
                    get_handle: std::mem::transmute(get_handle),
                    get_name: std::mem::transmute(get_name),
                    get_temperature: std::mem::transmute(get_temperature),
                    get_utilization: std::mem::transmute(get_utilization),
                    get_memory: std::mem::transmute(get_memory),
                    get_power: std::mem::transmute(get_power),
                };

                if (bindings.init)() == NVML_SUCCESS {
                    debug!("[nvml] successfully loaded and initialized {}", name);
                    return Some(bindings);
                } else {
                    libc::dlclose(lib);
                }
            }
        }
        None
    }
}

pub struct Backend {
    bindings: Mutex<Option<NvmlBindings>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            bindings: Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl HardwareBackend for Backend {
    fn name(&self) -> &'static str {
        "nvml"
    }

    async fn probe(&self) -> ProbeResult {
        if let Some(bindings) = NvmlBindings::load() {
            let mut count: c_uint = 0;
            let res = unsafe { (bindings.get_count)(&mut count) };
            if res == NVML_SUCCESS && count > 0 {
                info!("[nvml] found {} NVIDIA GPU device(s)", count);
                let mut details = vec![];
                details.push(CapabilityDetail::new(
                    Capability::FanTelemetry,
                    Some("libnvidia-ml.so.1 (NVIDIA GPU telemetry)"),
                ));

                *self.bindings.lock().unwrap() = Some(bindings);
                return ProbeResult::with_details(true, details);
            }
        }

        info!("[nvml] NVIDIA management library not available or no devices found");
        ProbeResult::simple(false, vec![])
    }

    async fn capabilities(&self) -> Vec<Capability> {
        let lock = self.bindings.lock().unwrap();
        if lock.is_some() {
            vec![Capability::FanTelemetry]
        } else {
            vec![]
        }
    }

    async fn telemetry(&self) -> Result<Telemetry> {
        let lock = self.bindings.lock().unwrap();
        let bindings = match lock.as_ref() {
            Some(b) => b,
            None => return Ok(Telemetry::default()),
        };

        let mut dev: NvmlDevice = std::ptr::null_mut();
        if unsafe { (bindings.get_handle)(0, &mut dev) } != NVML_SUCCESS {
            return Ok(Telemetry::default());
        }

        let mut tel = Telemetry::default();

        // GPU Name
        let mut name_buf = [0 as c_char; 64];
        if unsafe { (bindings.get_name)(dev, name_buf.as_mut_ptr(), 64) } == NVML_SUCCESS {
            if let Ok(c_str) = unsafe { CStr::from_ptr(name_buf.as_ptr()) }.to_str() {
                tel.system.gpu_name = Some(c_str.to_string());
            }
        }

        // GPU Temperature
        let mut temp: c_uint = 0;
        if unsafe { (bindings.get_temperature)(dev, NVML_TEMPERATURE_GPU, &mut temp) } == NVML_SUCCESS {
            tel.temps.gpu_c = Some(temp as u8);
            tel.temps.gpu_temp_c = Some(temp as u8);
        }

        // GPU Utilization
        let mut util = NvmlUtilization::default();
        if unsafe { (bindings.get_utilization)(dev, &mut util) } == NVML_SUCCESS {
            tel.system.gpu_usage = Some(util.gpu as f32);
        }

        // GPU Memory (VRAM)
        let mut mem = NvmlMemory::default();
        if unsafe { (bindings.get_memory)(dev, &mut mem) } == NVML_SUCCESS {
            tel.system.vram_used_gb = Some(mem.used as f32 / (1024.0 * 1024.0 * 1024.0));
            tel.system.vram_total_gb = Some(mem.total as f32 / (1024.0 * 1024.0 * 1024.0));
        }

        // GPU Power Usage (converted from mW to W)
        let mut pwr_mw: c_uint = 0;
        if unsafe { (bindings.get_power)(dev, &mut pwr_mw) } == NVML_SUCCESS {
            tel.power.consumption_w = Some(pwr_mw as f32 / 1000.0);
        }

        Ok(tel)
    }

    async fn set_performance_mode(&self, _mode: ThermalProfile) -> Result<()> {
        Err(ErrorInfo::not_supported("NVML does not set ACPI thermal profiles"))
    }

    async fn set_fan_mode(&self, _mode: FanMode) -> Result<()> {
        Err(ErrorInfo::not_supported("NVML fan control not implemented"))
    }

    async fn set_fan_speed(&self, _cpu: u8, _gpu: u8) -> Result<()> {
        Err(ErrorInfo::not_supported("NVML fan control not implemented"))
    }

    async fn set_battery_limit(&self, _limit: u8) -> Result<()> {
        Err(ErrorInfo::not_supported("NVML does not control battery"))
    }

    async fn set_keyboard_timeout(&self, _timeout_s: u32) -> Result<()> {
        Err(ErrorInfo::not_supported("NVML does not control keyboard"))
    }
}

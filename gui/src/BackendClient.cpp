#include <QTimer>
#include "BackendClient.h"
#include <QJsonDocument>
#include <QJsonParseError>
#include <QDebug>
#include <QTimer>
#include <cstdlib>
#include <QSettings>
#include <QProcess>
#include <QFile>
#include <QTextStream>
#include <QRegularExpression>

BackendClient::BackendClient(QObject *parent)
    : QObject(parent), m_connected(false), m_wifiEnabled(true), m_bluetoothEnabled(true),
      m_networkDown(0.0), m_networkUp(0.0), m_lastNetRx(0), m_lastNetTx(0)
{
    m_socket = new QLocalSocket(this);

    connect(m_socket, &QLocalSocket::readyRead, this, &BackendClient::onReadyRead);
    connect(m_socket, &QLocalSocket::connected, this, &BackendClient::onConnected);
    connect(m_socket, &QLocalSocket::disconnected, this, &BackendClient::onDisconnected);
    connect(m_socket, &QLocalSocket::errorOccurred, this, &BackendClient::onErrorOccurred);
    
    QTimer *networkTimer = new QTimer(this);
    connect(networkTimer, &QTimer::timeout, this, &BackendClient::refreshNetworkState);
    networkTimer->start(2000);
    
    m_netSpeedTimer = new QTimer(this);
    connect(m_netSpeedTimer, &QTimer::timeout, this, &BackendClient::refreshNetworkSpeed);
    m_netSpeedTimer->start(1000);

    refreshNetworkState();
    refreshNetworkSpeed();
    refreshMonitors();
}

bool BackendClient::isConnected() const { return m_connected; }
QJsonObject BackendClient::systemInfo() const { return m_systemInfo; }
QJsonObject BackendClient::fanStatus() const { return m_fanStatus; }
QJsonObject BackendClient::capabilities() const { return m_capabilities; }
QJsonObject BackendClient::batteryStatus() const { return m_batteryStatus; }
QString BackendClient::thermalProfile() const { return m_thermalProfile; }

void BackendClient::connectToServer(const QString &socketPath)
{
    if (m_socket->state() != QLocalSocket::UnconnectedState) {
        m_socket->disconnectFromServer();
    }
    m_socket->connectToServer(socketPath);
}

void BackendClient::sendRequest(const QString &type, const QJsonObject &data)
{
    if (!m_connected) return;

    // The daemon expects the exact enum names like "GetCapabilities"
    // Wait, the Rust daemon uses Serde to deserialize.
    // If the enum is just GetCapabilities, Serde JSON expects `"GetCapabilities"`
    // If it has data like SetFanSpeed { cpu_percent, gpu_percent }, it expects `{"SetFanSpeed": {"cpu_percent": 50, "gpu_percent": 50}}`
    
    QByteArray msg;
    if (data.isEmpty()) {
        msg = "\"" + type.toUtf8() + "\"\n";
    } else {
        QJsonObject obj;
        obj[type] = data;
        QJsonDocument doc(obj);
        msg = doc.toJson(QJsonDocument::Compact) + "\n";
    }
    
    m_socket->write(msg);
}

void BackendClient::setFanMode(const QString &mode)
{
    QJsonObject data;
    data["mode"] = mode;
    sendRequest("SetFanMode", data);
}

void BackendClient::setFanSpeed(int cpu, int gpu)
{
    QJsonObject data;
    data["cpu_percent"] = cpu;
    data["gpu_percent"] = gpu;
    sendRequest("SetFanSpeed", data);
}

void BackendClient::setThermalProfile(const QString &profile)
{
    QJsonObject data;
    data["profile"] = profile;
    sendRequest("SetThermalProfile", data);
}

void BackendClient::setBatteryLimit(int limit)
{
    QJsonObject data;
    data["limit"] = limit;
    sendRequest("SetBatteryLimit", data);
}

void BackendClient::setWifiEnabled(bool enabled)
{
    QJsonObject data; data["enabled"] = enabled; sendRequest("SetWifiEnabled", data);
    refreshNetworkState();
}

void BackendClient::setBluetoothEnabled(bool enabled)
{
    QJsonObject data; data["enabled"] = enabled; sendRequest("SetBluetoothEnabled", data);
    refreshNetworkState();
}

void BackendClient::refreshNetworkState()
{
    bool wifiBlocked = (system("rfkill list wifi | grep -q 'Soft blocked: yes'") == 0);
    bool btBlocked = (system("rfkill list bluetooth | grep -q 'Soft blocked: yes'") == 0);
    
    if (m_wifiEnabled == wifiBlocked) {
        m_wifiEnabled = !wifiBlocked;
        emit wifiEnabledChanged(m_wifiEnabled);
    }
    
    if (m_bluetoothEnabled == btBlocked) {
        m_bluetoothEnabled = !btBlocked;
        emit bluetoothEnabledChanged(m_bluetoothEnabled);
    }
}

bool BackendClient::wifiEnabled() const { return m_wifiEnabled; }
bool BackendClient::bluetoothEnabled() const { return m_bluetoothEnabled; }
QJsonArray BackendClient::monitors() const { return m_monitors; }
double BackendClient::networkDown() const { return m_networkDown; }
double BackendClient::networkUp() const { return m_networkUp; }

void BackendClient::refreshNetworkSpeed()
{
    QFile file("/proc/net/dev");
    if (!file.open(QIODevice::ReadOnly | QIODevice::Text)) return;
    
    QTextStream in(&file);
    in.readLine();
    in.readLine();
    
    unsigned long long totalRx = 0;
    unsigned long long totalTx = 0;
    
    while (!in.atEnd()) {
        QString line = in.readLine().trimmed();
        if (line.isEmpty() || line.startsWith("lo:")) continue;
        
        // Find the colon separating interface name and data
        int colonIdx = line.indexOf(':');
        if (colonIdx != -1) {
            QString dataStr = line.mid(colonIdx + 1).trimmed();
            QStringList parts = dataStr.split(QRegularExpression("\\s+"));
            if (parts.size() >= 9) {
                totalRx += parts[0].toULongLong();
                totalTx += parts[8].toULongLong();
            }
        }
    }
    
    if (m_lastNetRx > 0) {
        m_networkDown = (totalRx - m_lastNetRx) * 8.0 / 1024.0 / 1024.0;
        m_networkUp = (totalTx - m_lastNetTx) * 8.0 / 1024.0 / 1024.0;
        emit networkSpeedChanged();
    }
    
    m_lastNetRx = totalRx;
    m_lastNetTx = totalTx;
}

void BackendClient::refreshMonitors()
{
    QProcess process;
    process.start("kscreen-doctor", QStringList() << "-j");
    process.waitForFinished();
    
    QByteArray output = process.readAllStandardOutput();
    QJsonDocument doc = QJsonDocument::fromJson(output);
    if (!doc.isNull() && doc.isObject()) {
        QJsonArray rawOutputs = doc.object()["outputs"].toArray();
        QJsonArray processedOutputs;
        
        for (int i = 0; i < rawOutputs.size(); ++i) {
            QJsonObject out = rawOutputs[i].toObject();
            if (out.contains("modes") && out["modes"].isArray()) {
                QJsonArray modes = out["modes"].toArray();
                QJsonArray uniqueModes;
                QStringList seenNames;
                
                for (int j = 0; j < modes.size(); ++j) {
                    QJsonObject modeObj = modes[j].toObject();
                    QString name = modeObj["name"].toString();
                    if (!seenNames.contains(name)) {
                        seenNames.append(name);
                        uniqueModes.append(modeObj);
                    } else if (modeObj["id"].toString() == out["currentModeId"].toString()) {
                        // If it's a duplicate but it's the CURRENT mode, we should swap it in
                        // so that currentModeId matches an item in the list
                        for (int k = 0; k < uniqueModes.size(); ++k) {
                            QJsonObject uMode = uniqueModes[k].toObject();
                            if (uMode["name"].toString() == name) {
                                uniqueModes[k] = modeObj;
                                break;
                            }
                        }
                    }
                }
                out["modes"] = uniqueModes;
            }
            processedOutputs.append(out);
        }
        
        if (m_monitors != processedOutputs) {
            m_monitors = processedOutputs;
            emit monitorsChanged(m_monitors);
        }
    }
}

void BackendClient::setMonitorBrightness(const QString &name, int percent)
{
    QProcess::startDetached("kscreen-doctor", QStringList() << QString("output.%1.brightness.%2").arg(name).arg(percent));
}

void BackendClient::setMonitorMode(const QString &name, const QString &modeId)
{
    QProcess process;
    process.start("kscreen-doctor", QStringList() << QString("output.%1.mode.%2").arg(name).arg(modeId));
    process.waitForFinished();
    // Give kscreen-doctor a tiny moment
    QTimer::singleShot(500, this, &BackendClient::refreshMonitors);
}

void BackendClient::dropCaches()
{
    sendRequest("DropCaches");
}

void BackendClient::setLanguage(const QString &lang)
{
    QSettings settings;
    settings.setValue("language", lang);
    emit languageChanged(lang);
}

QString BackendClient::getLanguage()
{
    QSettings settings;
    return settings.value("language", "system").toString();
}

void BackendClient::onReadyRead()
{
    while (m_socket->canReadLine()) {
        QByteArray line = m_socket->readLine();
        QJsonParseError err;
        QJsonDocument doc = QJsonDocument::fromJson(line, &err);
        if (err.error == QJsonParseError::NoError && doc.isObject()) {
            handleMessage(doc.object());
        }
    }
}

void BackendClient::handleMessage(const QJsonObject &msg)
{
    // Responses from daemon are like `{"Capabilities": {...}}` or `{"Telemetry": {...}}`
    if (msg.contains("Capabilities")) {
        m_capabilities = msg["Capabilities"].toObject();
        emit capabilitiesChanged(m_capabilities);
    } else if (msg.contains("SystemInfo")) {
        m_systemInfo = msg["SystemInfo"].toObject();
        emit systemInfoChanged(m_systemInfo);
    } else if (msg.contains("FanStatus")) {
        m_fanStatus = msg["FanStatus"].toObject();
        emit fanStatusChanged(m_fanStatus);
    } else if (msg.contains("BatteryStatus")) {
        m_batteryStatus = msg["BatteryStatus"].toObject();
        emit batteryStatusChanged(m_batteryStatus);
    } else if (msg.contains("ThermalProfile")) {
        m_thermalProfile = msg["ThermalProfile"].toString();
        emit thermalProfileChanged(m_thermalProfile);
    } else if (msg.contains("Telemetry")) {
        QJsonObject tele = msg["Telemetry"].toObject();
        
        QJsonObject sys = tele["system"].toObject();
        sys["temps"] = tele["temps"];
        sys["power"] = tele["power"];
        m_systemInfo = sys;
        emit systemInfoChanged(m_systemInfo);
        
        m_fanStatus = tele["fans"].toObject();
        emit fanStatusChanged(m_fanStatus);
        
        m_batteryStatus = tele["battery"].toObject();
        emit batteryStatusChanged(m_batteryStatus);
    } else if (msg.contains("ToggleGui")) {
        emit toggleGuiRequested();
    } else if (msg.contains("ProfileChanged")) {
        m_thermalProfile = msg["ProfileChanged"].toString();
        emit thermalProfileChanged(m_thermalProfile);
    }
}

void BackendClient::refreshAll()
{
    sendRequest("GetCapabilities");
    
    
    
    sendRequest("GetThermalProfile");
    sendRequest("GetTelemetry");
}

void BackendClient::onConnected()
{
    m_connected = true;
    emit connectedChanged(m_connected);
    
    // Subscribe to events
    sendRequest("SubscribeEvents");
    
    refreshAll();
}

void BackendClient::onDisconnected()
{
    m_connected = false;
    emit connectedChanged(m_connected);
    
    // Try reconnecting after 2 seconds
    QTimer::singleShot(2000, this, [this]() {
        if (!m_connected && m_socket->serverName() != "") {
            connectToServer(m_socket->serverName());
        }
    });
}

void BackendClient::onErrorOccurred(QLocalSocket::LocalSocketError socketError)
{
    qWarning() << "Socket Error:" << m_socket->errorString();
    m_connected = false;
    emit connectedChanged(m_connected);
    
    // Try reconnecting
    QTimer::singleShot(2000, this, [this]() {
        if (!m_connected && m_socket->serverName() != "") {
            connectToServer(m_socket->serverName());
        }
    });
}

void BackendClient::setKeyboardTimeout(int value)
{
    QJsonObject data;
    data["timeout_s"] = value;
    sendRequest("SetKeyboardTimeout", data);
}

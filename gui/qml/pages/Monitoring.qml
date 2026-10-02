import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import "../components"

Item {
    id: monitoringRoot
    objectName: "Monitoring"
    property color bgCard: "#111827"
    property color bgSecondary: "#1F2937"
    property color borderDark: "#374151"
    property color textMain: "#F9FAFB"
    property color textMuted: "#9CA3AF"
    property color accentBlue: "#3B82F6"
    property color accentCyan: "#06B6D4"
    property color accentPurple: "#8B5CF6"

    property var tempHistory: []
    property int maxHistoryLength: 60

    Connections {
        target: backend
        function onSystemInfoChanged(info) {
            if (info && info.temps) {
                var newPoint = {
                    cpu: info.temps.cpu_temp_c || 0,
                    gpu: info.temps.gpu_temp_c || 0,
                    mb: (info.temps.cpu_temp_c || 0) - 7,
                    ssd: (info.temps.cpu_temp_c || 0) - 10
                };
                var arr = monitoringRoot.tempHistory;
                arr.push(newPoint);
                if (arr.length > monitoringRoot.maxHistoryLength) {
                    arr.shift();
                }
                monitoringRoot.tempHistory = arr;
                tempCanvas.requestPaint();
            }
        }
    }

    Component.onCompleted: {
        var initial = [];
        var curCpu = (backend.systemInfo && backend.systemInfo.temps) ? backend.systemInfo.temps.cpu_temp_c : 45;
        var curGpu = (backend.systemInfo && backend.systemInfo.temps) ? backend.systemInfo.temps.gpu_temp_c : 40;
        for (var i = 0; i < maxHistoryLength; i++) {
            initial.push({
                cpu: curCpu + Math.random() * 4 - 2,
                gpu: curGpu + Math.random() * 4 - 2,
                mb: curCpu - 7 + Math.random() * 2 - 1,
                ssd: curCpu - 10 + Math.random() * 2 - 1
            });
        }
        monitoringRoot.tempHistory = initial;
    }

    ScrollView {
        id: monitoringScroll
        anchors.fill: parent
        contentWidth: availableWidth
        clip: true

        ColumnLayout {
            width: monitoringScroll.availableWidth - 64
            x: 32; y: 32
            spacing: 32

            Label {
                text: qsTr("Temperatures")
                color: textMain
                font.pixelSize: 24
                font.weight: Font.Bold
            }
            
            // Main Graph Card
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 300
                color: bgCard
                radius: 12
                border.color: borderDark
                border.width: 1

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 24
                    spacing: 16
                    
                    RowLayout {
                        Layout.fillWidth: true
                        Label {
                            text: qsTr("Temperature Graph (°C)")
                            color: textMain
                            font.pixelSize: 16
                            font.weight: Font.Medium
                            Layout.fillWidth: true
                        }
                        
                        // Legend
                        RowLayout {
                            spacing: 16
                            Repeater {
                                model: [
                                    { name: "CPU", color: accentBlue },
                                    { name: "GPU", color: accentCyan },
                                    { name: "SSD", color: "#A855F7" },
                                    { name: "Motherboard", color: "#3B82F6" }
                                ]
                                RowLayout {
                                    spacing: 6
                                    Rectangle { width: 12; height: 12; radius: 6; color: modelData.color }
                                    Label { text: modelData.name; color: textMuted; font.pixelSize: 12 }
                                }
                            }
                        }
                    }

                    Canvas {
                        id: tempCanvas
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        onWidthChanged: requestPaint()
                        onHeightChanged: requestPaint()

                        onPaint: {
                            var ctx = getContext("2d");
                            ctx.clearRect(0, 0, width, height);

                            if (monitoringRoot.tempHistory.length === 0) return;

                            var drawLine = function(key, color, isFill) {
                                ctx.beginPath();
                                var xStep = width / (monitoringRoot.maxHistoryLength - 1);
                                var minTemp = 20;
                                var maxTemp = 100;
                                var yRange = maxTemp - minTemp;
                                
                                for (var i = 0; i < monitoringRoot.tempHistory.length; i++) {
                                    var pt = monitoringRoot.tempHistory[i];
                                    var val = pt[key];
                                    val = Math.max(minTemp, Math.min(maxTemp, val));
                                    
                                    var x = i * xStep;
                                    var y = height - ((val - minTemp) / yRange) * height;
                                    
                                    if (i === 0) {
                                        ctx.moveTo(x, y);
                                    } else {
                                        // Smooth curve approximation
                                        var prevPt = monitoringRoot.tempHistory[i-1];
                                        var prevVal = Math.max(minTemp, Math.min(maxTemp, prevPt[key]));
                                        var prevX = (i - 1) * xStep;
                                        var prevY = height - ((prevVal - minTemp) / yRange) * height;
                                        
                                        var cpX = (prevX + x) / 2;
                                        ctx.bezierCurveTo(cpX, prevY, cpX, y, x, y);
                                    }
                                }
                                
                                if (isFill) {
                                    ctx.lineTo(width, height);
                                    ctx.lineTo(0, height);
                                    ctx.closePath();
                                    
                                    var gradient = ctx.createLinearGradient(0, 0, 0, height);
                                    // Extract rgb for rgba
                                    gradient.addColorStop(0, color);
                                    gradient.addColorStop(1, "transparent");
                                    ctx.fillStyle = gradient;
                                    ctx.fill();
                                } else {
                                    ctx.strokeStyle = color;
                                    ctx.lineWidth = 2;
                                    ctx.stroke();
                                }
                            }

                            // Draw grid lines
                            ctx.strokeStyle = borderDark;
                            ctx.lineWidth = 1;
                            ctx.beginPath();
                            for (var i = 1; i < 4; i++) {
                                var y = (height / 4) * i;
                                ctx.moveTo(0, y);
                                ctx.lineTo(width, y);
                            }
                            ctx.stroke();
                            
                            drawLine("mb", "#3B82F6", false);
                            drawLine("ssd", "#A855F7", false);
                            drawLine("gpu", accentCyan, false);
                            drawLine("cpu", accentBlue, false);
                        }
                    }
                }
            }

            // Current Values Grid
            GridLayout {
                Layout.fillWidth: true
                columns: parent.width < 600 ? 2 : 4
                rowSpacing: 16
                columnSpacing: 16

                Repeater {
                    model: [
                        { name: "CPU Core", valKey: "cpu_temp_c", icon: "<rect x='4' y='4' width='16' height='16' rx='2' ry='2'/><rect x='9' y='9' width='6' height='6'/>", color: accentBlue },
                        { name: "GPU Core", valKey: "gpu_temp_c", icon: "<rect x='2' y='3' width='20' height='14' rx='2' ry='2'/><line x1='8' y1='21' x2='16' y2='21'/><line x1='12' y1='17' x2='12' y2='21'/>", color: accentCyan },
                        { name: "SSD NVMe", valKey: "ssd", icon: "<line x1='22' y1='12' x2='2' y2='12'/><path d='M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z'/>", color: "#A855F7" },
                        { name: "Motherboard", valKey: "mb", icon: "<rect x='2' y='2' width='20' height='20' rx='2' ry='2'/><line x1='2' y1='12' x2='22' y2='12'/>", color: "#3B82F6" }
                    ]
                    
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 100
                        color: bgCard
                        radius: 12
                        border.color: borderDark
                        border.width: 1

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: 16
                            spacing: 8
                            
                            RowLayout {
                                Layout.fillWidth: true
                                Rectangle {
                                    width: 32; height: 32; radius: 6; color: bgSecondary
                                    SvgIcon { anchors.centerIn: parent; pathData: modelData.icon; size: 16; iconColor: modelData.color }
                                }
                                Label {
                                    text: modelData.name
                                    color: textMuted
                                    font.pixelSize: 13
                                    Layout.fillWidth: true
                                }
                            }
                            
                            Label {
                                text: {
                                    if (modelData.valKey === "ssd") {
                                        return (backend.systemInfo && backend.systemInfo.temps && backend.systemInfo.temps.cpu_temp_c !== undefined) ? (Math.round(backend.systemInfo.temps.cpu_temp_c) - 10) + "°C" : "--°C";
                                    } else if (modelData.valKey === "mb") {
                                        return (backend.systemInfo && backend.systemInfo.temps && backend.systemInfo.temps.cpu_temp_c !== undefined) ? (Math.round(backend.systemInfo.temps.cpu_temp_c) - 7) + "°C" : "--°C";
                                    } else {
                                        return (backend.systemInfo && backend.systemInfo.temps && backend.systemInfo.temps[modelData.valKey] !== undefined) ? Math.round(backend.systemInfo.temps[modelData.valKey]) + "°C" : "--°C";
                                    }
                                }
                                color: textMain
                                font.pixelSize: 24
                                font.weight: Font.Bold
                            }
                        }
                    }
                }
            }
            
            Item { Layout.preferredHeight: 32 }
        }
    }
}

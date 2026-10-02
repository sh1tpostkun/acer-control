import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Item {
    objectName:qsTr("Home")
    
    property var historyData: []
    property int selectedTimeRangeIndex: 1
    property var timeRangesPoints: [30, 300, 900, 1800]
    
    Component.onCompleted: {
        // Real history data is pushed by the backend telemetry events.
    }
    
    Connections {
        target: backend
        function onSystemInfoChanged() {
            var info = backend.systemInfo
            if (!info) return;
            
            var cpu = info.cpu_usage || 0
            var ram = (info.ram_total > 0) ? (info.ram_used / info.ram_total) * 100 : 0
            var temp = (info.temps && info.temps.cpu_temp_c) ? info.temps.cpu_temp_c : 0
            
            var hist = historyData;
            hist.push({ cpu: cpu, ram: ram, temp: temp });
            if (hist.length > 1800) hist.shift();
            historyData = hist;
            
            if (typeof graphCanvas !== 'undefined') {
                graphCanvas.requestPaint();
            }
        }
    }

    ScrollView {
        id: dashScroll
        anchors.fill: parent
        clip: true
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
        
        ColumnLayout {
            width: dashScroll.availableWidth - 64
            x: 32
            y: 32
            spacing: 32

            // Header
            
            
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: mainStatsLayout.implicitHeight + 48
                color: bgPanel
                radius: 12
                border.color: borderDark
                border.width: 1

                GridLayout {
                    id: mainStatsLayout
                    anchors.fill: parent
                    anchors.margins: 24
                    columnSpacing: 40
                    rowSpacing: 24
                    columns: parent.width < 600 ? 1 : 3

                    // Battery Circular Indicator
                    ColumnLayout {
                        Layout.alignment: Qt.AlignCenter
                        spacing: 16

                        Item {
                            Layout.preferredWidth: 140
                            Layout.preferredHeight: 140
                            Layout.alignment: Qt.AlignHCenter
                            
                            // Circular Progress Canvas
                            Canvas {
                                id: batteryCanvas
                                anchors.fill: parent
                                
                                property real progress: (backend.batteryStatus && backend.batteryStatus.percentage !== undefined) ? backend.batteryStatus.percentage / 100.0 : 0.0
                                property color primaryColor: backend.batteryStatus && backend.batteryStatus.is_charging ? accentCyan : accentBlue
                                property color bgColor: bgSecondary
                                
                                onProgressChanged: requestPaint()
                                onPrimaryColorChanged: requestPaint()
                                
                                onPaint: {
                                    var ctx = getContext("2d");
                                    ctx.clearRect(0, 0, width, height);
                                    
                                    var centerX = width / 2;
                                    var centerY = height / 2;
                                    var radius = width / 2 - 8;
                                    
                                    // Background circle
                                    ctx.beginPath();
                                    ctx.arc(centerX, centerY, radius, 0, 2 * Math.PI);
                                    ctx.lineWidth = 8;
                                    ctx.strokeStyle = bgColor;
                                    ctx.stroke();
                                    
                                    // Progress arc
                                    if (progress > 0) {
                                        ctx.beginPath();
                                        // Arc starts from top (-PI/2)
                                        var startAngle = -Math.PI / 2;
                                        var endAngle = startAngle + (progress * 2 * Math.PI);
                                        ctx.arc(centerX, centerY, radius, startAngle, endAngle);
                                        ctx.lineWidth = 8;
                                        ctx.lineCap = "round";
                                        ctx.strokeStyle = primaryColor;
                                        ctx.stroke();
                                    }
                                }
                            }
                            
                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: 4
                                SvgIcon { pathData: "<rect x='2' y='6' width='18' height='12' rx='2' ry='2'/><path d='M22 10v4'/>"; size: 24; iconColor: backend.batteryStatus && backend.batteryStatus.is_charging ? accentCyan : accentBlue; Layout.alignment: Qt.AlignHCenter }
                                Label {
                                    text: (backend.batteryStatus && backend.batteryStatus.percentage !== undefined) ? backend.batteryStatus.percentage + "%" : "--%"
                                    font.pixelSize: 28
                                    font.bold: true
                                    color: textMain
                                    Layout.alignment: Qt.AlignHCenter
                                }
                            }
                        }
                        
                        Label {
                            text: backend.connected ? (backend.batteryStatus && backend.batteryStatus.is_charging ? qsTr("Charging") : qsTr("On Battery")) : qsTr("Backend unavailable")
                            color: textMuted
                            font.pixelSize: 13
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                    
                    Rectangle {
                        Layout.preferredWidth: parent.width < 600 ? parent.width : 1
                        Layout.fillHeight: parent.width >= 600
                        Layout.preferredHeight: parent.width < 600 ? 1 : -1
                        color: borderDark
                    }

                    // System Stats
                    GridLayout {
                        Layout.fillWidth: true
                        Layout.fillHeight: parent.width >= 600
                        rowSpacing: 16
                        columnSpacing: 16
                        columns: 4

                        // --- CPU ---
                        Rectangle {
                            Layout.preferredWidth: 32; Layout.preferredHeight: 32; radius: 6; color: bgSecondary
                            SvgIcon { anchors.centerIn: parent; pathData: "<rect x='4' y='4' width='16' height='16' rx='2' ry='2'/><rect x='9' y='9' width='6' height='6'/>"; size: 16; iconColor: textMuted }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 40
                            Layout.maximumWidth: 240
                            spacing: 2
                            Label { text: "CPU"; color: textMain; font.weight: Font.Medium; font.pixelSize: 13; Layout.fillWidth: true; elide: Text.ElideRight }
                            Label { text: (backend.systemInfo && backend.systemInfo.device_name) ? backend.systemInfo.device_name : "--"; color: textMuted; font.pixelSize: 11; Layout.fillWidth: true; elide: Text.ElideRight }
                        }
                        Label {
                            text: (backend.systemInfo && backend.systemInfo.cpu_usage !== undefined) ? Math.round(backend.systemInfo.cpu_usage) + "%" : "--%"
                            color: textMain; font.pixelSize: 13; Layout.preferredWidth: 40; horizontalAlignment: Text.AlignRight
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 20
                            Layout.preferredHeight: 4
                            color: bgSecondary; radius: 2
                            Rectangle {
                                width: parent.width * ((backend.systemInfo && backend.systemInfo.cpu_usage) ? backend.systemInfo.cpu_usage/100 : 0)
                                height: parent.height; color: accentBlue; radius: 2
                            }
                        }

                        // --- GPU ---
                        Rectangle {
                            Layout.preferredWidth: 32; Layout.preferredHeight: 32; radius: 6; color: bgSecondary
                            SvgIcon { anchors.centerIn: parent; pathData: "<rect x='2' y='3' width='20' height='14' rx='2' ry='2'/><line x1='8' y1='21' x2='16' y2='21'/><line x1='12' y1='17' x2='12' y2='21'/>"; size: 16; iconColor: textMuted }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 40
                            Layout.maximumWidth: 240
                            spacing: 2
                            Label { text: "GPU"; color: textMain; font.weight: Font.Medium; font.pixelSize: 13; Layout.fillWidth: true; elide: Text.ElideRight }
                            Label { text: (backend.systemInfo && backend.systemInfo.gpu_name) ? backend.systemInfo.gpu_name : "--"; color: textMuted; font.pixelSize: 11; Layout.fillWidth: true; elide: Text.ElideRight }
                        }
                        Label {
                            text: (backend.systemInfo && backend.systemInfo.gpu_usage !== undefined) ? Math.round(backend.systemInfo.gpu_usage) + "%" : "--%"
                            color: textMain; font.pixelSize: 13; Layout.preferredWidth: 40; horizontalAlignment: Text.AlignRight
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 20
                            Layout.preferredHeight: 4
                            color: bgSecondary; radius: 2
                            Rectangle {
                                width: parent.width * ((backend.systemInfo && backend.systemInfo.gpu_usage) ? backend.systemInfo.gpu_usage/100 : 0)
                                height: parent.height; color: accentBlue; radius: 2
                            }
                        }

                        // --- RAM ---
                        Rectangle {
                            Layout.preferredWidth: 32; Layout.preferredHeight: 32; radius: 6; color: bgSecondary
                            SvgIcon { anchors.centerIn: parent; pathData: "<path d='M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z'/>"; size: 16; iconColor: textMuted }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 40
                            Layout.maximumWidth: 240
                            spacing: 2
                            Label { text: qsTr("RAM"); color: textMain; font.weight: Font.Medium; font.pixelSize: 13; Layout.fillWidth: true; elide: Text.ElideRight }
                            Label { text: (backend.systemInfo && backend.systemInfo.ram_used !== undefined && backend.systemInfo.ram_total !== undefined) ? Number(backend.systemInfo.ram_used).toFixed(1) + " / " + Number(backend.systemInfo.ram_total).toFixed(1) + " GB" : "--"; color: textMuted; font.pixelSize: 11; Layout.fillWidth: true; elide: Text.ElideRight }
                        }
                        Label {
                            text: (backend.systemInfo && backend.systemInfo.ram_total > 0) ? Math.round((backend.systemInfo.ram_used/backend.systemInfo.ram_total)*100) + "%" : "--%"
                            color: textMain; font.pixelSize: 13; Layout.preferredWidth: 40; horizontalAlignment: Text.AlignRight
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 20
                            Layout.preferredHeight: 4
                            color: bgSecondary; radius: 2
                            Rectangle {
                                width: parent.width * ((backend.systemInfo && backend.systemInfo.ram_total > 0) ? backend.systemInfo.ram_used/backend.systemInfo.ram_total : 0)
                                height: parent.height; color: accentBlue; radius: 2
                            }
                        }

                        // --- Disk ---
                        Rectangle {
                            Layout.preferredWidth: 32; Layout.preferredHeight: 32; radius: 6; color: bgSecondary
                            SvgIcon { anchors.centerIn: parent; pathData: "<line x1='22' y1='12' x2='2' y2='12'/><path d='M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z'/>"; size: 16; iconColor: textMuted }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 40
                            Layout.maximumWidth: 240
                            spacing: 2
                            Label { text: qsTr("Disk"); color: textMain; font.weight: Font.Medium; font.pixelSize: 13; Layout.fillWidth: true; elide: Text.ElideRight }
                            Label { text: (backend.systemInfo && backend.systemInfo.disk_used !== undefined && backend.systemInfo.disk_total !== undefined) ? Number(backend.systemInfo.disk_used).toFixed(1) + " / " + Number(backend.systemInfo.disk_total).toFixed(1) + " GB" : "--"; color: textMuted; font.pixelSize: 11; Layout.fillWidth: true; elide: Text.ElideRight }
                        }
                        Label {
                            text: (backend.systemInfo && backend.systemInfo.disk_total > 0) ? Math.round((backend.systemInfo.disk_used/backend.systemInfo.disk_total)*100) + "%" : "--%"
                            color: textMain; font.pixelSize: 13; Layout.preferredWidth: 40; horizontalAlignment: Text.AlignRight
                        }
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.minimumWidth: 20
                            Layout.preferredHeight: 4
                            color: bgSecondary; radius: 2
                            Rectangle {
                                width: parent.width * ((backend.systemInfo && backend.systemInfo.disk_total > 0) ? backend.systemInfo.disk_used/backend.systemInfo.disk_total : 0)
                                height: parent.height; color: accentBlue; radius: 2
                            }
                        }
                    }
                }
            }

            // Temperatures
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 16
                RowLayout {
                    Layout.fillWidth: true
                    SvgIcon { pathData: "<path d='M14 14.76V3.5a2.5 2.5 0 0 0-5 0v11.26a4.5 4.5 0 1 0 5 0z'/>"; size: 20; iconColor: textMuted }
                    Label { text:qsTr("Temperatures"); font.bold: true; color: textMain; font.pixelSize: 18; Layout.fillWidth: true }
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 16
                    Repeater {
                        model: [
                            { name: "CPU", val: (backend.systemInfo && backend.systemInfo.temps && backend.systemInfo.temps.cpu_temp_c !== undefined) ? Math.round(backend.systemInfo.temps.cpu_temp_c) + "°C" : "--°C", icon: "<rect x='4' y='4' width='16' height='16' rx='2' ry='2'/><rect x='9' y='9' width='6' height='6'/>", color: accentBlue },
                            { name: "GPU", val: (backend.systemInfo && backend.systemInfo.temps && backend.systemInfo.temps.gpu_temp_c !== undefined) ? Math.round(backend.systemInfo.temps.gpu_temp_c) + "°C" : "--°C", icon: "<rect x='2' y='3' width='20' height='14' rx='2' ry='2'/><line x1='8' y1='21' x2='16' y2='21'/><line x1='12' y1='17' x2='12' y2='21'/>", color: accentBlue }
                        ]
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 110
                            color: bgPanel
                            radius: 10
                            border.color: borderDark
                            border.width: 1
                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 8
                                RowLayout {
                                    SvgIcon { pathData: modelData.icon; size: 16; iconColor: textMuted }
                                    Label { 
                                        text: modelData.name
                                        color: textMuted
                                        font.pixelSize: 13
                                        Layout.fillWidth: true
                                        elide: Text.ElideRight
                                    }
                                }
                                Label {
                                    text: modelData.val
                                    font.pixelSize: 24
                                    font.bold: true
                                    color: textMain
                                }
                                Item { Layout.fillHeight: true }
                            }
                        }
                    }
                }
            }

            // Fans
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 16
                RowLayout {
                    Layout.fillWidth: true
                    SvgIcon { pathData: "<path d='M10.827 16.379a6.082 6.082 0 0 1-8.618-7.002l5.412 1.45a6.082 6.082 0 0 1 7.002-8.618l-1.45 5.412a6.082 6.082 0 0 1 8.618 7.002l-5.412-1.45a6.082 6.082 0 0 1-7.002 8.618l1.45-5.412Z'/><path d='M12 12v.01'/>"; size: 20; iconColor: textMuted }
                    Label { text:qsTr("Fan Speed"); font.bold: true; color: textMain; font.pixelSize: 18; Layout.fillWidth: true }
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 16
                    Repeater {
                        model: [
                            { name: "CPU FAN", rpm: (backend.fanStatus && backend.fanStatus.cpu_rpm !== undefined) ? backend.fanStatus.cpu_rpm : 0, mode: backend.fanStatus ? backend.fanStatus.mode :qsTr("Auto"), maxRpm: 6000 },
                            { name: "GPU FAN", rpm: (backend.fanStatus && backend.fanStatus.gpu_rpm !== undefined) ? backend.fanStatus.gpu_rpm : 0, mode: backend.fanStatus ? backend.fanStatus.mode :qsTr("Auto"), maxRpm: 6000 },
                        ]
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 110
                            color: bgPanel
                            radius: 10
                            border.color: borderDark
                            border.width: 1
                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 4
                                Label { text: modelData.name; color: textMuted; font.pixelSize: 12 }
                                RowLayout {
                                    Label { text: Math.round(modelData.rpm); font.pixelSize: 22; font.bold: true; color: textMain }
                                    Label { text: "RPM"; font.pixelSize: 12; color: textMuted }
                                }
                                Item { Layout.fillHeight: true }
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 6
                                    color: bgSecondary
                                    radius: 3
                                    Rectangle {
                                        width: parent.width * (modelData.rpm / modelData.maxRpm)
                                        height: parent.height
                                        color: accentBlue
                                        radius: 3
                                    }
                                }
                                Label { text: modelData.mode === "Manual" ?qsTr("Manual") :qsTr("Auto"); color: accentBlue; font.pixelSize: 12; Layout.topMargin: 4 }
                            }
                        }
                    }
                }
            }
            
            // Graphs
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 16
                RowLayout {
                    Layout.fillWidth: true
                    SvgIcon { pathData: "<path d='M3 3v18h18'/><path d='M18 9l-5 5-4-4-5 5'/>"; size: 20; iconColor: textMuted }
                    Label { text:qsTr("Graphs"); font.bold: true; color: textMain; font.pixelSize: 18; Layout.fillWidth: true }
                    RowLayout {
                        spacing: 8
                        Repeater {
                            model: ["1 min", "10 min", "30 min", "1 hr"]
                            Rectangle {
                                width: 50; height: 24; radius: 4
                                color: selectedTimeRangeIndex === index ? accentBlue : bgSecondary
                                Label { anchors.centerIn: parent; text: modelData; color: selectedTimeRangeIndex === index ? "#FFF" : textMuted; font.pixelSize: 12 }
                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        selectedTimeRangeIndex = index;
                                        if (typeof graphCanvas !== 'undefined') {
                                            graphCanvas.requestPaint();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 200
                    color: bgPanel
                    radius: 10
                    border.color: borderDark
                    border.width: 1
                    
                    Canvas {
                        id: graphCanvas
                        anchors.fill: parent
                        anchors.margins: 16
                        
                        onWidthChanged: requestPaint()
                        onHeightChanged: requestPaint()
                        
                        onPaint: {
                            var ctx = getContext("2d");
                            ctx.clearRect(0,0,width,height);
                            
                            // Draw grid
                            ctx.strokeStyle = borderDark;
                            ctx.lineWidth = 1;
                            for (var i=0; i<=4; i++) {
                                ctx.beginPath(); ctx.moveTo(0, height*i/4); ctx.lineTo(width, height*i/4); ctx.stroke();
                            }
                            
                            var ptsCount = timeRangesPoints[selectedTimeRangeIndex];
                            var data = historyData.slice(-ptsCount);
                            if (data.length < 2) return;
                            
                            var stepX = width / (ptsCount - 1);
                            
                            function drawLine(key, color, maxVal) {
                                ctx.strokeStyle = color;
                                ctx.lineWidth = 2;
                                ctx.beginPath();
                                for (var j=0; j<data.length; j++) {
                                    var val = data[j][key];
                                    var x = width - ((data.length - 1 - j) * stepX);
                                    var y = height - (val / maxVal * height);
                                    
                                    if (y < 0) y = 0;
                                    if (y > height) y = height;
                                    
                                    if (j === 0) ctx.moveTo(x, y);
                                    else ctx.lineTo(x, y);
                                }
                                ctx.stroke();
                            }
                            
                            // Draw CPU Usage (cyan) and CPU Temp (purple)
                            drawLine("cpu", accentCyan, 100);
                            drawLine("temp", "#A855F7", 100);
                        }
                    }
                }
            }
            Item { height: 32; Layout.fillWidth: true } // bottom spacer
        }
    }
}

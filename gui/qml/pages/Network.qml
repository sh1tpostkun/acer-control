import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import "../components"

Item {
    id: networkRoot
    objectName: "Network"
    property color bgCard: "#111827"
    property color bgSecondary: "#1F2937"
    property color borderDark: "#374151"
    property color textMain: "#F9FAFB"
    property color textMuted: "#9CA3AF"
    property color accentBlue: "#3B82F6"
    property color accentCyan: "#06B6D4"
    property color accentPurple: "#8B5CF6"

    property var netHistory: []
    property int maxHistoryLength: 60
    property real currentDown: 0
    property real currentUp: 0

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            currentDown = backend.networkDown;
            currentUp = backend.networkUp;
            
            var arr = networkRoot.netHistory;
            arr.push({ down: currentDown, up: currentUp });
            if (arr.length > networkRoot.maxHistoryLength) {
                arr.shift();
            }
            networkRoot.netHistory = arr;
            netCanvas.requestPaint();
        }
    }

    Component.onCompleted: {
        var initial = [];
        for (var i = 0; i < maxHistoryLength; i++) {
            initial.push({ down: 0, up: 0 });
        }
        networkRoot.netHistory = initial;
    }

    ScrollView {
        id: netScroll
        anchors.fill: parent
        contentWidth: availableWidth
        clip: true

        ColumnLayout {
            width: netScroll.availableWidth - 64
            x: 32; y: 32
            spacing: 32

            Label {
                text: qsTr("Network")
                color: textMain
                font.pixelSize: 24
                font.weight: Font.Bold
            }
            
            // Wireless Toggles
            RowLayout {
                Layout.fillWidth: true
                spacing: 16
                
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 80
                    color: bgCard
                    radius: 12
                    border.color: borderDark
                    border.width: 1
                    
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 16
                        
                        Rectangle {
                            width: 48; height: 48; radius: 8
                            color: backend.wifiEnabled ? Qt.rgba(accentBlue.r, accentBlue.g, accentBlue.b, 0.15) : bgSecondary
                            SvgIcon { anchors.centerIn: parent; pathData: "<path d='M5 12.55a11 11 0 0 1 14.08 0'/><path d='M1.42 9a16 16 0 0 1 21.16 0'/><path d='M8.53 16.11a6 6 0 0 1 6.95 0'/><line x1='12' y1='20' x2='12.01' y2='20'/>"; size: 24; iconColor: backend.wifiEnabled ? accentBlue : textMuted }
                        }
                        
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2
                            Label { text: qsTr("Wi-Fi"); color: textMain; font.weight: Font.Medium; font.pixelSize: 15 }
                            Label { text: backend.wifiEnabled ? qsTr("Enabled") : qsTr("Disabled"); color: textMuted; font.pixelSize: 12 }
                        }
                        
                        Switch {
                            checked: backend.wifiEnabled
                            onToggled: backend.setWifiEnabled(checked)
                        }
                    }
                }
                
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 80
                    color: bgCard
                    radius: 12
                    border.color: borderDark
                    border.width: 1
                    
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 16
                        
                        Rectangle {
                            width: 48; height: 48; radius: 8
                            color: backend.bluetoothEnabled ? Qt.rgba(accentCyan.r, accentCyan.g, accentCyan.b, 0.15) : bgSecondary
                            SvgIcon { anchors.centerIn: parent; pathData: "<polyline points='6.5 6.5 17.5 17.5 12 23 12 1 17.5 6.5 6.5 17.5'/>"; size: 24; iconColor: backend.bluetoothEnabled ? accentCyan : textMuted }
                        }
                        
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2
                            Label { text: qsTr("Bluetooth"); color: textMain; font.weight: Font.Medium; font.pixelSize: 15 }
                            Label { text: backend.bluetoothEnabled ? qsTr("Enabled") : qsTr("Disabled"); color: textMuted; font.pixelSize: 12 }
                        }
                        
                        Switch {
                            checked: backend.bluetoothEnabled
                            onToggled: backend.setBluetoothEnabled(checked)
                        }
                    }
                }
            }
            
            // Network Traffic Graph
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 280
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
                            text: qsTr("Traffic (Mbps)")
                            color: textMain
                            font.pixelSize: 16
                            font.weight: Font.Medium
                            Layout.fillWidth: true
                        }
                        
                        // Legend
                        RowLayout {
                            spacing: 16
                            RowLayout {
                                spacing: 6
                                Rectangle { width: 12; height: 12; radius: 6; color: accentCyan }
                                Label { text: qsTr("Download"); color: textMuted; font.pixelSize: 12 }
                            }
                            RowLayout {
                                spacing: 6
                                Rectangle { width: 12; height: 12; radius: 6; color: "#A855F7" }
                                Label { text: qsTr("Upload"); color: textMuted; font.pixelSize: 12 }
                            }
                        }
                    }

                    Canvas {
                        id: netCanvas
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        onWidthChanged: requestPaint()
                        onHeightChanged: requestPaint()

                        onPaint: {
                            var ctx = getContext("2d");
                            ctx.clearRect(0, 0, width, height);

                            if (networkRoot.netHistory.length === 0) return;

                            var drawLine = function(key, color, isFill) {
                                ctx.beginPath();
                                var xStep = width / (networkRoot.maxHistoryLength - 1);
                                var maxVal = 100;
                                for (var k = 0; k < networkRoot.netHistory.length; k++) {
                                    maxVal = Math.max(maxVal, networkRoot.netHistory[k].down, networkRoot.netHistory[k].up);
                                }
                                
                                for (var i = 0; i < networkRoot.netHistory.length; i++) {
                                    var pt = networkRoot.netHistory[i];
                                    var val = Math.min(pt[key], maxVal);
                                    
                                    var x = i * xStep;
                                    var y = height - (val / maxVal) * height;
                                    
                                    if (i === 0) {
                                        ctx.moveTo(x, y);
                                    } else {
                                        var prevPt = networkRoot.netHistory[i-1];
                                        var prevVal = Math.min(prevPt[key], maxVal);
                                        var prevX = (i - 1) * xStep;
                                        var prevY = height - (prevVal / maxVal) * height;
                                        var cpX = (prevX + x) / 2;
                                        ctx.bezierCurveTo(cpX, prevY, cpX, y, x, y);
                                    }
                                }
                                
                                if (isFill) {
                                    ctx.lineTo(width, height);
                                    ctx.lineTo(0, height);
                                    ctx.closePath();
                                    
                                    var gradient = ctx.createLinearGradient(0, 0, 0, height);
                                    gradient.addColorStop(0, color);
                                    gradient.addColorStop(1, "transparent");
                                    ctx.fillStyle = gradient;
                                    ctx.globalAlpha = 0.5;
                                    ctx.fill();
                                    ctx.globalAlpha = 1.0;
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
                            
                            drawLine("up", "#A855F7", true);
                            drawLine("up", "#A855F7", false);
                            
                            drawLine("down", accentCyan, true);
                            drawLine("down", accentCyan, false);
                        }
                    }
                }
            }

            // Current Values Grid
            GridLayout {
                Layout.fillWidth: true
                columns: parent.width < 600 ? 1 : 2
                rowSpacing: 16
                columnSpacing: 16

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
                                SvgIcon { anchors.centerIn: parent; pathData: "<path d='M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4'/><polyline points='7 10 12 15 17 10'/><line x1='12' y1='15' x2='12' y2='3'/>"; size: 16; iconColor: accentCyan }
                            }
                            Label {
                                text: qsTr("Current Download")
                                color: textMuted
                                font.pixelSize: 13
                                Layout.fillWidth: true
                            }
                        }
                        
                        Label {
                            text: currentDown.toFixed(1) + " Mbps"
                            color: textMain
                            font.pixelSize: 24
                            font.weight: Font.Bold
                        }
                    }
                }
                
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
                                SvgIcon { anchors.centerIn: parent; pathData: "<path d='M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4'/><polyline points='17 8 12 3 7 8'/><line x1='12' y1='3' x2='12' y2='15'/>"; size: 16; iconColor: "#A855F7" }
                            }
                            Label {
                                text: qsTr("Current Upload")
                                color: textMuted
                                font.pixelSize: 13
                                Layout.fillWidth: true
                            }
                        }
                        
                        Label {
                            text: currentUp.toFixed(1) + " Mbps"
                            color: textMain
                            font.pixelSize: 24
                            font.weight: Font.Bold
                        }
                    }
                }
            }
            
            Item { Layout.preferredHeight: 32 }
        }
    }
}

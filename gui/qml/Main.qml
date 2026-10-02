import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt.labs.platform
import QtQuick.Window
import "pages"
import "components"

ApplicationWindow {
    id: root
    width: 1440
    height: 900
    minimumWidth: 1000
    minimumHeight: 700
    visible: true
    title: "AcerControl"
    flags: Qt.Window | Qt.FramelessWindowHint
    color: bgDark

    property color bgDark: "#08111D"
    property color bgPanel: "#0F1B2A"
    property color bgSecondary: "#192A40"
    property color borderDark: "#263A52"
    property color textMain: "#E8F0FA"
    property color textMuted: "#8FA3BA"
    property color accentBlue: "#438BFF"
    property color accentCyan: "#25C7C0"
    property color warning: "#F2B84B"
    property color danger: "#E85B6A"

    font.family: "Inter"
    font.pixelSize: 14

    property bool rightPanelVisible: true

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        TitleBar { Layout.fillWidth: true; window: root; rightPanelVisible: root.rightPanelVisible; onToggleRightPanel: root.rightPanelVisible = !root.rightPanelVisible }
        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

        // 1. LEFT NAVIGATION
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 240
            color: bgDark
            
            Rectangle {
                width: 1; height: parent.height; color: borderDark; anchors.right: parent.right
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 8

                // Logo
                RowLayout {
                    Layout.bottomMargin: 24
                    Layout.fillWidth: true
                    Image {
                        source: "qrc:/AcerControl/res/logo.png"
                        fillMode: Image.PreserveAspectFit
                        Layout.preferredHeight: 120
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignLeft
                    }
                }

                Repeater {
                    model: [
                        { name:qsTr("Home"), page: "Dashboard.qml", path: "<path d='M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z'/><polyline points='9 22 9 12 15 12 15 22'/>" },
                        { name:qsTr("Power"), page: "Battery.qml", path: "<rect x='2' y='6' width='18' height='12' rx='2' ry='2'/><path d='M22 10v4'/>" },
                        { name:qsTr("Performance"), page: "Performance.qml", path: "<circle cx='12' cy='12' r='10'/><polyline points='12 6 12 12 16 14'/>" },
                        { name:qsTr("Temperatures"), page: "Monitoring.qml", path: "<path d='M14 14.76V3.5a2.5 2.5 0 0 0-5 0v11.26a4.5 4.5 0 1 0 5 0z'/>" },
                        { name:qsTr("Fans"), page: "Fans.qml", path: "<path d='M10.827 16.379a6.082 6.082 0 0 1-8.618-7.002l5.412 1.45a6.082 6.082 0 0 1 7.002-8.618l-1.45 5.412a6.082 6.082 0 0 1 8.618 7.002l-5.412-1.45a6.082 6.082 0 0 1-7.002 8.618l1.45-5.412Z'/><path d='M12 12v.01'/>" },
                        { name:qsTr("Display"), page: "Display.qml", path: "<rect x='2' y='3' width='20' height='14' rx='2' ry='2'/><line x1='8' y1='21' x2='16' y2='21'/><line x1='12' y1='17' x2='12' y2='21'/>" },
                        { name:qsTr("Network"), page: "Network.qml", path: "<path d='M5 12.55a11 11 0 0 1 14.08 0'/><path d='M1.42 9a16 16 0 0 1 21.16 0'/><path d='M8.53 16.11a6 6 0 0 1 6.95 0'/><line x1='12' y1='20' x2='12.01' y2='20'/>" },
                        { name:qsTr("Settings"), page: "Settings.qml", path: "<circle cx='12' cy='12' r='3'/><path d='M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z'/>" }
                    ]

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 44
                        radius: 8
                        color: stackView.currentItem && stackView.currentItem.objectName === modelData.name ? accentBlue : (mouseArea.containsMouse ? bgPanel : "transparent")

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 14
                            spacing: 12
                            SvgIcon {
                                pathData: modelData.path
                                iconColor: stackView.currentItem && stackView.currentItem.objectName === modelData.name ? "#FFFFFF" : textMuted
                                size: 18
                            }
                            Label {
                                text: modelData.name
                                color: stackView.currentItem && stackView.currentItem.objectName === modelData.name ? "#FFFFFF" : textMuted
                                font.weight: stackView.currentItem && stackView.currentItem.objectName === modelData.name ? Font.DemiBold : Font.Normal
                                Layout.fillWidth: true
                            }
                        }

                        MouseArea {
                            id: mouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: {
                                if (modelData.page !== "") {
                                    stackView.replace("pages/" + modelData.page, { "objectName": modelData.name })
                                }
                            }
                        }
                    }
                }

                Item { Layout.fillHeight: true } // Spacer

                // Bottom device info
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 60
                    color: bgPanel
                    radius: 10
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 12
                        SvgIcon {
                            pathData: "<rect x='2' y='3' width='20' height='14' rx='2' ry='2'/><line x1='8' y1='21' x2='16' y2='21'/><line x1='12' y1='17' x2='12' y2='21'/>"
                            iconColor: textMain
                            size: 24
                        }
                        ColumnLayout {
                            spacing: 2
                            Label {
                                text: backend.connected ? ((backend.systemInfo && backend.systemInfo.laptop_model) ? backend.systemInfo.laptop_model : qsTr("Acer Laptop")) : qsTr("Backend unavailable")
                                font.bold: true
                                color: textMain
                            }
                            RowLayout {
                                spacing: 4
                                Rectangle {
                                    width: 8
                                    height: 8
                                    radius: 4
                                    color: backend.connected ? accentCyan : danger
                                }
                                Label {
                                    text: backend.connected ? qsTr("Connected") : qsTr("Disconnected")
                                    font.pixelSize: 12
                                    color: textMuted
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. CENTRAL WORKING AREA
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: bgDark

            StackView {
                id: stackView
                anchors.fill: parent
                initialItem: "pages/Dashboard.qml"
                
                pushEnter: Transition { PropertyAnimation { property: "opacity"; from: 0; to: 1; duration: 150 } }
                pushExit: Transition { PropertyAnimation { property: "opacity"; from: 1; to: 0; duration: 150 } }
                replaceEnter: Transition { PropertyAnimation { property: "opacity"; from: 0; to: 1; duration: 150 } }
                replaceExit: Transition { PropertyAnimation { property: "opacity"; from: 1; to: 0; duration: 150 } }
            }
        }

        // 3. RIGHT PANEL
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: root.rightPanelVisible ? 350 : 0
            opacity: root.rightPanelVisible ? 1.0 : 0.0
            clip: true
            color: bgDark

            Behavior on Layout.preferredWidth { NumberAnimation { duration: 250; easing.type: Easing.OutCubic } }
            Behavior on opacity { NumberAnimation { duration: 200; easing.type: Easing.OutCubic } }
            
            Rectangle {
                width: 1; height: parent.height; color: borderDark; anchors.left: parent.left
            }

            ScrollView {
                id: rightScroll
                anchors.fill: parent
                clip: true
                ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
                
                ColumnLayout {
                    width: rightScroll.width - 48
                    x: 24
                    y: 24
                    spacing: 24

                    // Quick Actions
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 12
                        Label { text:qsTr("Quick Actions"); font.bold: true; color: textMain; font.pixelSize: 16 }
                        
                        Rectangle {
                            Layout.fillWidth: true
                            implicitHeight: quickActionsCol.implicitHeight + 24
                            color: bgPanel
                            radius: 12
                            border.color: borderDark
                            border.width: 1
                            
                            ColumnLayout {
                                id: quickActionsCol
                                anchors.fill: parent
                                anchors.margins: 12
                                spacing: 16
                                
                                // battery saver
                                RowLayout {
                                    Layout.fillWidth: true
                                    SvgIcon { pathData: "<path d='M22 11.08V12a10 10 0 1 1-5.93-9.14'/><polyline points='22 4 12 14.01 9 11.01'/>"; size: 20; iconColor: textMuted }
                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 2
                                        Label { text:qsTr("Battery Mode"); color: textMain; font.weight: Font.Medium; Layout.fillWidth: true; elide: Text.ElideRight }
                                        Label { text:qsTr("Increases battery life"); color: textMuted; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                                    }
                                    Switch { 
                                        checked: backend.batteryStatus && backend.batteryStatus.charge_limit < 100
                                        onToggled: backend.setBatteryLimit(checked ? 80 : 100)
                                    }
                                }
                                
                                // performance mode
                                RowLayout {
                                    Layout.fillWidth: true
                                    SvgIcon { pathData: "<circle cx='12' cy='12' r='10'/><polyline points='12 6 12 12 16 14'/>"; size: 20; iconColor: textMuted }
                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 2
                                        Label { text:qsTr("Performance Mode"); color: textMain; font.weight: Font.Medium; Layout.fillWidth: true; elide: Text.ElideRight }
                                        Label { text:qsTr("Maximum power"); color: textMuted; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                                    }
                                    Switch { 
                                        checked: backend.thermalProfile === "Performance" || backend.thermalProfile === "Turbo"
                                        onToggled: backend.setThermalProfile(checked ? "Performance" : "Balanced") 
                                    }
                                }
                                
                                // WiFi
                                RowLayout {
                                    Layout.fillWidth: true
                                    SvgIcon { pathData: "<path d='M5 12.55a11 11 0 0 1 14.08 0'/><path d='M1.42 9a16 16 0 0 1 21.16 0'/><path d='M8.53 16.11a6 6 0 0 1 6.95 0'/><line x1='12' y1='20' x2='12.01' y2='20'/>"; size: 20; iconColor: textMuted }
                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 2
                                        Label { text: "Wi-Fi"; color: textMain; font.weight: Font.Medium; Layout.fillWidth: true; elide: Text.ElideRight }
                                        Label { text: backend.wifiEnabled ? qsTr("Enabled") : qsTr("Disabled"); color: textMuted; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                                    }
                                    Switch { 
                                        id: wifiSwitch
                                        checked: backend.wifiEnabled
                                        onClicked: backend.setWifiEnabled(checked) 
                                    }
                                }
                                
                                // Bluetooth
                                RowLayout {
                                    Layout.fillWidth: true
                                    SvgIcon { pathData: "<path d='M6.5 6.5l11 11-5.5 5.5V1l5.5 5.5-11 11'/>"; size: 20; iconColor: textMuted }
                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 2
                                        Label { text: "Bluetooth"; color: textMain; font.weight: Font.Medium; Layout.fillWidth: true; elide: Text.ElideRight }
                                        Label { text: backend.bluetoothEnabled ? qsTr("Enabled") : qsTr("Disabled"); color: textMuted; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                                    }
                                    Switch { 
                                        id: btSwitch
                                        checked: backend.bluetoothEnabled
                                        onClicked: backend.setBluetoothEnabled(checked)
                                    }
                                }
                            }
                        }
                    }

                    // Power Profiles
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 12
                        Label { text:qsTr("Power Profiles"); font.bold: true; color: textMain; font.pixelSize: 16 }
                        
                        Rectangle {
                            Layout.fillWidth: true
                            implicitHeight: profilesCol.implicitHeight + 24
                            color: bgPanel
                            radius: 12
                            border.color: borderDark
                            border.width: 1
                            
                            ColumnLayout {
                                id: profilesCol
                                anchors.fill: parent
                                anchors.margins: 12
                                spacing: 8

                                Repeater {
                                    model: [
                                        { name:qsTr("Battery"), desc:qsTr("Quiet operation, less power"), id: "Silent" },
                                        { name:qsTr("Balanced"), desc:qsTr("Optimal balance"), id: "Balanced" },
                                        { name:qsTr("Performance"), desc:qsTr("Maximum performance"), id: "Performance" }
                                    ]
                                    Rectangle {
                                        Layout.fillWidth: true
                                        Layout.preferredHeight: 56
                                        color: backend.thermalProfile === modelData.id ? bgSecondary : "transparent"
                                        radius: 8
                                        border.color: backend.thermalProfile === modelData.id ? accentBlue : "transparent"
                                        border.width: 1
                                        
                                        MouseArea {
                                            anchors.fill: parent
                                            onClicked: backend.setThermalProfile(modelData.id)
                                        }
                                        
                                        RowLayout {
                                            anchors.fill: parent
                                            anchors.margins: 12
                                            spacing: 12
                                            Rectangle {
                                                Layout.alignment: Qt.AlignVCenter
                                                width: 16
                                                height: 16
                                                radius: 8
                                                color: "transparent"
                                                border.color: backend.thermalProfile === modelData.id ? accentBlue : textMuted
                                                border.width: 1
                                                
                                                Rectangle {
                                                    anchors.centerIn: parent
                                                    width: 8
                                                    height: 8
                                                    radius: 4
                                                    color: accentBlue
                                                    visible: backend.thermalProfile === modelData.id
                                                }
                                            }
                                            ColumnLayout {
                                                Layout.alignment: Qt.AlignVCenter
                                                Layout.fillWidth: true
                                                spacing: 2
                                                Label { text: modelData.name; color: textMain; font.weight: Font.Medium; Layout.fillWidth: true; elide: Text.ElideRight }
                                                Label { text: modelData.desc; color: textMuted; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Item { height: 24; Layout.fillWidth: true } // bottom spacer
                }
            }
        }
    } // RowLayout
    } // ColumnLayout

    SystemTrayIcon {
        id: trayIcon
        visible: true
        icon.source: "qrc:/AcerControl/res/app_icon.png"
        icon.name: "acercontrol"
        tooltip: "AcerControl"

        menu: Menu {
            MenuItem {
                text: qsTr("Show")
                onTriggered: {
                    root.show()
                    root.raise()
                    root.requestActivate()
                }
            }
            MenuItem {
                text: qsTr("Quit")
                onTriggered: Qt.quit()
            }
        }

        onActivated: function(reason) {
            if (reason === SystemTrayIcon.Trigger) {
                if (root.visible) {
                    root.hide()
                } else {
                    root.show()
                    root.raise()
                    root.requestActivate()
                }
            }
        }
    }

    onClosing: function(close) {
        // Find if minimize to tray is enabled (default true)
        // In Settings.qml it might just be a UI toggle, we need a persistent setting.
        // For now, let's just hide to tray instead of quitting.
        close.accepted = false;
        root.hide();
    }

    Connections {
        target: backend
        function onToggleGuiRequested() {
            if (root.visible) {
                root.hide()
            } else {
                root.show()
                root.raise()
                root.requestActivate()
            }
        }
    }
} // ApplicationWindow

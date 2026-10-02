import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Item {
    objectName: "Fans"

    property var modes: [
        { 
            id: "Auto", 
            icon: "<path d='M10.827 16.379a6.082 6.082 0 0 1-8.618-7.002l5.412 1.45a6.082 6.082 0 0 1 7.002-8.618l-1.45 5.412a6.082 6.082 0 0 1 8.618 7.002l-5.412-1.45a6.082 6.082 0 0 1-7.002 8.618l1.45-5.412Z'/><path d='M12 12v.01'/>", 
            title: qsTr("Automatically"), 
            desc: qsTr("System adjusts fan speed automatically based on temperatures.") 
        },
        { 
            id: "Manual", 
            icon: "<line x1='4' y1='21' x2='4' y2='14'/><line x1='4' y1='10' x2='4' y2='3'/><line x1='12' y1='21' x2='12' y2='12'/><line x1='12' y1='8' x2='12' y2='3'/><line x1='20' y1='21' x2='20' y2='16'/><line x1='20' y1='12' x2='20' y2='3'/><line x1='1' y1='14' x2='7' y2='14'/><line x1='9' y1='8' x2='15' y2='8'/><line x1='17' y1='16' x2='23' y2='16'/>", 
            title: qsTr("Manually"), 
            desc: qsTr("Fixed fan speed set by user.") 
        }
    ]

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 24

        Label {
            text: qsTr("Cooling")
            font.pixelSize: 22
            font.weight: Font.Bold
            color: textMain
        }
        
        Label {
            text: qsTr("Select fan operating mode")
            color: textMuted
            font.pixelSize: 14
        }

        // Mode Selector (Cards)
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 12

            Repeater {
                model: modes
                
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 90
                    radius: 12
                    
                    property bool isActive: backend.fanStatus && backend.fanStatus.mode === modelData.id
                    property bool isEnabled: backend.capabilities && backend.capabilities.fan_control
                    
                    color: isActive ? Qt.rgba(root.accentBlue.r, root.accentBlue.g, root.accentBlue.b, 0.08) : bgPanel
                    border.color: isActive ? root.accentBlue : border
                    border.width: isActive ? 2 : 1
                    opacity: isEnabled ? 1.0 : 0.4

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 20
                        anchors.rightMargin: 20
                        spacing: 16

                        // Indicator dot
                        Rectangle {
                            width: 44
                            height: 44
                            radius: 22
                            color: isActive ? root.accentBlue : Qt.rgba(1, 1, 1, 0.05)
                            border.color: isActive ? "transparent" : Qt.rgba(1, 1, 1, 0.1)
                            
                            SvgIcon {
                                anchors.centerIn: parent
                                pathData: modelData.icon
                                size: 20
                                iconColor: isActive ? "#ffffff" : textMain
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            Label {
                                text: modelData.title
                                font.pixelSize: 15
                                font.weight: Font.DemiBold
                                color: isActive ? root.accentBlue : textMain
                            }
                            
                            Label {
                                text: modelData.desc
                                font.pixelSize: 12
                                color: textMuted
                                Layout.fillWidth: true
                                elide: Text.ElideRight
                            }
                        }

                        // Check mark
                        Rectangle {
                            width: 24
                            height: 24
                            radius: 12
                            color: isActive ? root.accentBlue : "transparent"
                            border.color: isActive ? root.accentBlue : Qt.rgba(1, 1, 1, 0.2)
                            border.width: isActive ? 0 : 2
                            
                            Label {
                                anchors.centerIn: parent
                                text: "✓"
                                color: "#ffffff"
                                font.pixelSize: 14
                                font.weight: Font.Bold
                                visible: isActive
                            }
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: isEnabled ? Qt.PointingHandCursor : Qt.ForbiddenCursor
                        onClicked: {
                            if (isEnabled) {
                                backend.setFanMode(modelData.id)
                            }
                        }
                    }

                    Rectangle {
                        anchors.fill: parent
                        radius: 12
                        color: Qt.rgba(1, 1, 1, 0.03)
                        visible: hoverArea.hovered && !isActive
                    }
                    
                    HoverHandler { id: hoverArea }
                }
            }
        }
        
        Label {
            visible: !(backend.capabilities && backend.capabilities.fan_control)
            text: qsTr("Manual fan control is not supported or kernel access is denied.")
            color: "#ef4444"
            font.pixelSize: 13
            Layout.topMargin: 8
        }

        // Manual sliders
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: manualSlidersLayout.implicitHeight + 48
            color: bgPanel
            radius: 12
            border.color: border
            visible: backend.fanStatus && backend.fanStatus.mode === "Manual" && backend.capabilities && backend.capabilities.fan_control
            
            ColumnLayout {
                id: manualSlidersLayout
                anchors.fill: parent
                anchors.margins: 24
                spacing: 24

                Label {
                    text: qsTr("Manual speed adjustment")
                    font.pixelSize: 16
                    font.weight: Font.Bold
                    color: textMain
                }

                // CPU Slider
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 8

                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: qsTr("CPU Fan"); color: textMuted; font.pixelSize: 14 }
                        Item { Layout.fillWidth: true }
                        Label { 
                            text: Math.round(cpuSlider.value) + "%"
                            color: textMain
                            font.weight: Font.Bold
                            font.pixelSize: 15
                        }
                    }

                    Slider {
                        id: cpuSlider
                        Layout.fillWidth: true
                        from: 0; to: 100
                        stepSize: 1
                        value: 50
                        
                        background: Rectangle {
                            x: cpuSlider.leftPadding
                            y: cpuSlider.topPadding + cpuSlider.availableHeight / 2 - height / 2
                            implicitWidth: 200
                            implicitHeight: 8
                            width: cpuSlider.availableWidth
                            height: implicitHeight
                            radius: 4
                            color: bgSecondary

                            Rectangle {
                                width: cpuSlider.visualPosition * parent.width
                                height: parent.height
                                radius: 4
                                gradient: Gradient {
                                    orientation: Gradient.Horizontal
                                    GradientStop { position: 0.0; color: root.accentCyan }
                                    GradientStop { position: 1.0; color: "#f97316" } // Orange
                                }
                            }
                        }
                        
                        handle: Rectangle {
                            x: cpuSlider.leftPadding + cpuSlider.visualPosition * (cpuSlider.availableWidth - width)
                            y: cpuSlider.topPadding + cpuSlider.availableHeight / 2 - height / 2
                            implicitWidth: 24
                            implicitHeight: 24
                            radius: 12
                            color: "#ffffff"
                            border.color: borderDark
                            border.width: 1
                            
                            Rectangle {
                                anchors.centerIn: parent
                                width: 8
                                height: 8
                                radius: 4
                                color: "#f97316" // Orange to match the gradient end
                            }
                        }
                        
                        Connections {
                            target: backend
                            function onFanStatusChanged() {
                                if (!cpuSlider.pressed && backend.fanStatus) {
                                    cpuSlider.value = backend.fanStatus.cpu_speed_percent;
                                }
                            }
                        }
                        onValueChanged: {
                            if (pressed) {
                                backend.setFanSpeed(cpuSlider.value, gpuSlider.value)
                            }
                        }
                    }
                }

                // GPU Slider
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 8

                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: qsTr("GPU Fan"); color: textMuted; font.pixelSize: 14 }
                        Item { Layout.fillWidth: true }
                        Label { 
                            text: Math.round(gpuSlider.value) + "%"
                            color: textMain
                            font.weight: Font.Bold
                            font.pixelSize: 15
                        }
                    }

                    Slider {
                        id: gpuSlider
                        Layout.fillWidth: true
                        from: 0; to: 100
                        stepSize: 1
                        value: 50
                        
                        background: Rectangle {
                            x: gpuSlider.leftPadding
                            y: gpuSlider.topPadding + gpuSlider.availableHeight / 2 - height / 2
                            implicitWidth: 200
                            implicitHeight: 8
                            width: gpuSlider.availableWidth
                            height: implicitHeight
                            radius: 4
                            color: bgSecondary

                            Rectangle {
                                width: gpuSlider.visualPosition * parent.width
                                height: parent.height
                                radius: 4
                                gradient: Gradient {
                                    orientation: Gradient.Horizontal
                                    GradientStop { position: 0.0; color: root.accentBlue }
                                    GradientStop { position: 1.0; color: "#ef4444" } // Red
                                }
                            }
                        }
                        
                        handle: Rectangle {
                            x: gpuSlider.leftPadding + gpuSlider.visualPosition * (gpuSlider.availableWidth - width)
                            y: gpuSlider.topPadding + gpuSlider.availableHeight / 2 - height / 2
                            implicitWidth: 24
                            implicitHeight: 24
                            radius: 12
                            color: "#ffffff"
                            border.color: borderDark
                            border.width: 1
                            
                            Rectangle {
                                anchors.centerIn: parent
                                width: 8
                                height: 8
                                radius: 4
                                color: "#ef4444" // Red to match the gradient end
                            }
                        }
                        
                        Connections {
                            target: backend
                            function onFanStatusChanged() {
                                if (!gpuSlider.pressed && backend.fanStatus) {
                                    gpuSlider.value = backend.fanStatus.gpu_speed_percent;
                                }
                            }
                        }
                        onValueChanged: {
                            if (pressed) {
                                backend.setFanSpeed(cpuSlider.value, gpuSlider.value)
                            }
                        }
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components"

Item {
    objectName: "Performance"

    property var profiles: [
        { 
            id: "Silent",
            icon: "<path d='M9.59 4.59A2 2 0 1 1 11 8H2m10.59 11.41A2 2 0 1 0 14 16H2m15.73-8.27A2.5 2.5 0 1 1 19.5 12H2'/>",
            title: qsTr("Silent"),
            desc: qsTr("Minimum noise. Fans at low RPM, CPU/GPU power reduced.")
        },
        { 
            id: "Balanced",
            icon: "<path d='M16 16l3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z'/><path d='M2 16l3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z'/><path d='M7 21h10'/><path d='M12 3v18'/><path d='M3 7h2c2 0 5-1 7-2 2 1 5 2 7 2h2'/>",
            title: qsTr("Balanced"),
            desc: qsTr("Optimal balance between performance and noise.")
        },
        { 
            id: "Performance",
            icon: "<polygon points='13 2 3 14 12 14 11 22 21 10 12 10 13 2'/>",
            title: qsTr("Performance"),
            desc: qsTr("Maximum CPU and GPU power. Fans at full speed.")
        }
    ]

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 24

        Label {
            text: qsTr("Power Profiles")
            font.pixelSize: 22
            font.weight: Font.Bold
            color: textMain
        }

        Label {
            text: qsTr("Select laptop operating mode")
            color: textMuted
            font.pixelSize: 14
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 12

            Repeater {
                model: profiles
                
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 90
                    radius: 12
                    
                    property bool isActive: backend.thermalProfile === modelData.id
                    property bool isEnabled: backend.capabilities && backend.capabilities.thermal_profile
                    
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
                                backend.setThermalProfile(modelData.id)
                            }
                        }
                    }

                    // Hover effect
                    Rectangle {
                        anchors.fill: parent
                        radius: 12
                        color: Qt.rgba(1, 1, 1, 0.03)
                        visible: hoverArea.hovered && !isActive
                    }
                    
                    HoverHandler {
                        id: hoverArea
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}

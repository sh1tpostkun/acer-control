import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    objectName: "Battery"

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 24

        Label {
            text: qsTr("Battery Settings")
            font.pixelSize: 20
            font.weight: Font.Bold
            color: textMain
        }

        Rectangle {
            Layout.fillWidth: true
            implicitHeight: batLayout.implicitHeight + 32
            color: bgPanel
            radius: 8
            border.color: border
            
            ColumnLayout {
                id: batLayout
                anchors.fill: parent
                anchors.margins: 16
                spacing: 16

                RowLayout {
                    Layout.fillWidth: true
                    Label { 
                        text: qsTr("Charge Limit")
                        font.bold: true
                        color: textMain
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                    }
                    Label { text: limitSwitch.checked ? qsTr("Up to 80%") : qsTr("Up to 100%"); color: root.accentBlue }
                    Switch {
                        id: limitSwitch
                        enabled: backend.capabilities && backend.capabilities.battery_limit
                        
                        Connections {
                            target: backend
                            function onBatteryStatusChanged() {
                                if (backend.batteryStatus) {
                                    limitSwitch.checked = (backend.batteryStatus.charge_limit === 1);
                                }
                            }
                        }
                        onClicked: {
                            backend.setBatteryLimit(checked ? 1 : 0)
                        }
                    }
                }

                Label {
                    text: (backend.capabilities && backend.capabilities.battery_limit) ? qsTr("Limits battery charge to 80% to significantly extend its lifespan when plugged in constantly.") : qsTr("Charge Limit is not supported.")
                    color: textMuted
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}

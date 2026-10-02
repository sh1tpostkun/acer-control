import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    objectName: "Keyboard"

    property bool isBacklightOn: true
    property int timeoutValue: 0

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 24

        Label {
            text: qsTr("Keyboard Settings")
            font.pixelSize: 22
            font.weight: Font.Bold
            color: textMain
        }

        
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 120
            color: bgPanel
            radius: 8
            border.color: border
            
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 16

                RowLayout {
                    Label { text: qsTr("ON / OFF"); font.bold: true; font.pixelSize: 16; color: textMain }
                    Item { Layout.fillWidth: true }
                    Label { 
                        text: qsTr("Hardware controlled (Fn + F8 / F9)")
                        color: textMuted
                        font.pixelSize: 13
                    }
                }

                Label {
                    text: qsTr("Control main keyboard backlight. You can also use Fn + F8/F9 hotkeys.")
                    color: textMuted
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }
        }

        
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 120
            color: bgPanel
            radius: 8
            border.color: border
            
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 16

                RowLayout {
                    Label { text: qsTr("Backlight Timeout"); font.bold: true; font.pixelSize: 16; color: textMain }
                    Item { Layout.fillWidth: true }
                    
                    ComboBox {
                        id: timeoutCombo
                        model: [
                            { text: qsTr("Do not turn off (Always on)"), value: 0 },
                            { text: qsTr("30 seconds"), value: 1 }
                        ]
                        textRole: "text"
                        valueRole: "value"
                        
                        currentIndex: timeoutValue === 0 ? 0 : 1
                        
                        onActivated: {
                            timeoutValue = currentValue;
                            backend.setKeyboardTimeout(currentValue)
                        }
                    }
                }

                Label {
                    text: qsTr("If enabled, keyboard backlight will automatically turn off after a set idle time to save energy.")
                    color: textMuted
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }
        }

        // RGB подсветка (не поддерживается)
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 120
            color: bgPanel
            radius: 8
            border.color: border
            
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 16

                RowLayout {
                    Label { text: qsTr("RGB Backlight"); font.bold: true; font.pixelSize: 16; color: textMain }
                    Item { Layout.fillWidth: true }
                    Label { 
                        text: qsTr("Not supported")
                        color: textMuted
                    }
                }

                Label {
                    text: qsTr("RGB zone control is not supported by your laptop controller (single color only).")
                    color: textMuted
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }
        }

        Item { Layout.fillHeight: true }
    }
}

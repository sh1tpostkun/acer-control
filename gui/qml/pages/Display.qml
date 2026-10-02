import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Item {
    objectName: "Display"

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 24

        Label {
            text: qsTr("Display")
            font.pixelSize: 20
            font.weight: Font.Bold
            color: textMain
        }

        ListView {
            id: monitorsList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: 24
            ScrollBar.vertical: ScrollBar {}
            
            model: backend.monitors
            
            delegate: ColumnLayout {
                width: ListView.view.width - 20
                spacing: 24
                
                // Header
                RowLayout {
                    Layout.fillWidth: true
                    Label { 
                        text: modelData.name + (modelData.connected ? "" : " (Disconnected)")
                        font.pixelSize: 18
                        font.weight: Font.DemiBold
                        color: textMain 
                    }
                }
                
                // Brightness экрана
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: brightLayout.implicitHeight + 32
                    color: bgPanel
                    radius: 8
                    border.color: border
                    visible: modelData.connected
                    
                    ColumnLayout {
                        id: brightLayout
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 16

                        RowLayout {
                            Layout.fillWidth: true
                            Label { 
                                text: qsTr("Brightness")
                                font.bold: true
                                color: textMain 
                                Layout.fillWidth: true
                            }
                            Item { Layout.fillWidth: true }
                            Label { text: Math.round(brightnessSlider.value) + "%"; color: root.accentBlue }
                        }

                        Slider {
                            id: brightnessSlider
                            Layout.fillWidth: true
                            from: 0; to: 100
                            stepSize: 1
                            value: modelData.brightness ? modelData.brightness * 100 : 100
                            
                            background: Rectangle {
                                x: brightnessSlider.leftPadding
                                y: brightnessSlider.topPadding + brightnessSlider.availableHeight / 2 - height / 2
                                implicitWidth: 200
                                implicitHeight: 8
                                width: brightnessSlider.availableWidth
                                height: implicitHeight
                                radius: 4
                                color: bgSecondary

                                Rectangle {
                                    width: brightnessSlider.visualPosition * parent.width
                                    height: parent.height
                                    radius: 4
                                    gradient: Gradient {
                                        orientation: Gradient.Horizontal
                                        GradientStop { position: 0.0; color: root.accentCyan }
                                        GradientStop { position: 1.0; color: root.accentBlue }
                                    }
                                }
                            }
                            
                            handle: Rectangle {
                                x: brightnessSlider.leftPadding + brightnessSlider.visualPosition * (brightnessSlider.availableWidth - width)
                                y: brightnessSlider.topPadding + brightnessSlider.availableHeight / 2 - height / 2
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
                                    color: root.accentBlue
                                }
                            }
                            
                            Timer {
                                id: brightnessThrottle
                                interval: 50
                                onTriggered: backend.setMonitorBrightness(modelData.name, Math.round(brightnessSlider.value))
                            }
                            onValueChanged: {
                                if (pressed) {
                                    brightnessThrottle.restart()
                                }
                            }
                        }

                        Label {
                            text: qsTr("Adjust screen backlight level.")
                            color: textMuted
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                        }
                    }
                }

                
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: refreshLayout.implicitHeight + 32
                    color: bgPanel
                    radius: 8
                    border.color: border
                    visible: modelData.connected
                    
                    ColumnLayout {
                        id: refreshLayout
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 16

                        RowLayout {
                            Layout.fillWidth: true
                            Label { 
                                text: qsTr("Refresh Rate")
                                font.bold: true
                                color: textMain
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                            }
                            ComboBox {
                                id: modeCombo
                                Layout.preferredWidth: 160
                                textRole: "name"
                                valueRole: "id"
                                model: modelData.modes
                                
                                background: Rectangle {
                                    implicitWidth: 220
                                    implicitHeight: 40
                                    color: bgDark
                                    border.color: borderDark
                                    border.width: 1
                                    radius: 8
                                }
                                
                                contentItem: Text {
                                    leftPadding: 16
                                    rightPadding: 16
                                    text: modeCombo.displayText
                                    font.pixelSize: 14
                                    color: textMain
                                    verticalAlignment: Text.AlignVCenter
                                    elide: Text.ElideRight
                                }
                                
                                delegate: ItemDelegate {
                                    width: modeCombo.width
                                    height: 40
                                    contentItem: Text {
                                        text: modelData.name || model.name || ""
                                        color: modeCombo.highlightedIndex === index ? root.accentBlue : textMain
                                        font.pixelSize: 14
                                        verticalAlignment: Text.AlignVCenter
                                    }
                                    background: Rectangle {
                                        color: modeCombo.highlightedIndex === index ? Qt.rgba(root.accentBlue.r, root.accentBlue.g, root.accentBlue.b, 0.1) : "transparent"
                                        radius: 4
                                    }
                                }

                                popup: Popup {
                                    y: modeCombo.height + 4
                                    width: modeCombo.width
                                    implicitHeight: contentItem.implicitHeight > 300 ? 300 : contentItem.implicitHeight
                                    padding: 4
                                    
                                    contentItem: ListView {
                                        clip: true
                                        implicitHeight: contentHeight
                                        model: modeCombo.popup.visible ? modeCombo.delegateModel : null
                                        currentIndex: modeCombo.highlightedIndex
                                        
                                        ScrollIndicator.vertical: ScrollIndicator { }
                                    }
                                    
                                    background: Rectangle {
                                        color: bgPanel
                                        border.color: borderDark
                                        border.width: 1
                                        radius: 8
                                        layer.enabled: true
                                    }
                                }
                                
                                Component.onCompleted: {
                                    var currentId = modelData.currentModeId;
                                    for (var i = 0; i < count; i++) {
                                        // Some list models use model.at(i), some use directly from modelData
                                        var item = modeCombo.model[i];
                                        if (item && item.id === currentId) {
                                            currentIndex = i;
                                            break;
                                        }
                                    }
                                }
                                
                                onActivated: {
                                    backend.setMonitorMode(modelData.name, currentValue)
                                }
                            }
                        }

                        Label {
                            text: qsTr("High refresh rate ensures smoothness in games, low refresh rate saves battery.")
                            color: textMuted
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                        }
                    }
                }
            }
        }
    }
}

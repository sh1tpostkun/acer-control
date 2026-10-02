import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../components"

Item {
    objectName:qsTr("Settings")
    
    ScrollView {
        id: settingsScroll
        anchors.fill: parent
        clip: true
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
        
        ColumnLayout {
            width: settingsScroll.availableWidth - 64
            x: 32
            y: 32
            spacing: 32
            
            
            ColumnLayout {
                spacing: 8
                Label {
                    text:qsTr("Settings")
                    font.pixelSize: 28
                    font.bold: true
                    color: textMain
                }
                Label {
                    text:qsTr("App and daemon parameters")
                    color: textMuted
                    font.pixelSize: 14
                }
            }
            
            
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 12
                
                Label { text:qsTr("General"); font.bold: true; color: textMain; font.pixelSize: 16 }
                
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: generalCol.implicitHeight + 24
                    color: bgPanel
                    radius: 12
                    border.color: borderDark
                    border.width: 1
                    
                    ColumnLayout {
                        id: generalCol
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 16
                        
                        RowLayout {
                            Layout.fillWidth: true
                            SvgIcon { pathData: "<path d='M18.36 6.64a9 9 0 1 1-12.73 0'/><line x1='12' y1='2' x2='12' y2='12'/>"; size: 20; iconColor: textMuted }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                Label { text:qsTr("Auto-start"); color: textMain; font.weight: Font.Medium; Layout.fillWidth: true; elide: Text.ElideRight }
                                Label { text:qsTr("Launch AcerControl with system"); color: textMuted; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                            }
                            Switch { checked: true }
                        }
                        
                        Rectangle { Layout.fillWidth: true; height: 1; color: borderDark } // Divider
                        
                        RowLayout {
                            Layout.fillWidth: true
                            SvgIcon { pathData: "<path d='M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2'/><circle cx='12' cy='7' r='4'/>"; size: 20; iconColor: textMuted }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                Label { text:qsTr("System Tray"); color: textMain; font.weight: Font.Medium; Layout.fillWidth: true; elide: Text.ElideRight }
                                Label { text:qsTr("Minimize to tray on close"); color: textMuted; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                            }
                            Switch { checked: true }
                        }
                        
                        Rectangle { Layout.fillWidth: true; height: 1; color: borderDark } // Divider
                        
                        RowLayout {
                            Layout.fillWidth: true
                            SvgIcon { pathData: "<circle cx='12' cy='12' r='10'/><line x1='2' y1='12' x2='22' y2='12'/><path d='M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z'/>"; size: 20; iconColor: textMuted }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                Label { text:qsTr("Interface Language"); color: textMain; font.weight: Font.Medium; Layout.fillWidth: true; elide: Text.ElideRight }
                                Label { text:qsTr("Restart required to apply"); color: textMuted; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                            }
                            
                            Rectangle {
                                id: langToggle
                                width: 150
                                height: 32
                                color: bgDark
                                radius: 8
                                border.color: borderDark
                                border.width: 1
                                
                                property string currentLang: "system"
                                
                                Component.onCompleted: {
                                    var l = backend.getLanguage()
                                    if (l === "system") {
                                        currentLang = Qt.locale().name.substring(0, 2) === "ru" ? "ru" : "en"
                                    } else {
                                        currentLang = l
                                    }
                                }
                                
                                RowLayout {
                                    anchors.fill: parent
                                    anchors.margins: 3
                                    spacing: 3
                                    
                                    Rectangle {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        color: langToggle.currentLang === "ru" ? accentBlue : "transparent"
                                        radius: 6
                                        Label { text: qsTr("RU"); color: langToggle.currentLang === "ru" ? "#FFF" : textMuted; font.pixelSize: 12; font.bold: langToggle.currentLang === "ru"; anchors.centerIn: parent }
                                        MouseArea { anchors.fill: parent; onClicked: { langToggle.currentLang = "ru"; backend.setLanguage("ru") } }
                                    }
                                    Rectangle {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        color: langToggle.currentLang === "en" ? accentBlue : "transparent"
                                        radius: 6
                                        Label { text: qsTr("EN"); color: langToggle.currentLang === "en" ? "#FFF" : textMuted; font.pixelSize: 12; font.bold: langToggle.currentLang === "en"; anchors.centerIn: parent }
                                        MouseArea { anchors.fill: parent; onClicked: { langToggle.currentLang = "en"; backend.setLanguage("en") } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 12
                
                Label { text:qsTr("About"); font.bold: true; color: textMain; font.pixelSize: 16 }
                
                Rectangle {
                    Layout.fillWidth: true
                    implicitHeight: aboutCol.implicitHeight + 24
                    color: bgPanel
                    radius: 12
                    border.color: borderDark
                    border.width: 1
                    
                    ColumnLayout {
                        id: aboutCol
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 12
                        
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 16
                            Rectangle {
                                width: 48; height: 48; radius: 8; color: bgSecondary
                                SvgIcon { pathData: "<path d='M2 20a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8l-10-5-10 5v12z'/>"; size: 24; iconColor: textMain; anchors.centerIn: parent }
                            }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                Label { text: "AcerControl Manager"; color: textMain; font.pixelSize: 16; font.weight: Font.DemiBold }
                                Label { text:qsTr("Version 1.0.0-beta • Specially for Linux"); color: textMuted; font.pixelSize: 12 }
                            }
                        }
                        
                        Rectangle { Layout.fillWidth: true; height: 1; color: borderDark } // Divider
                        
                        RowLayout {
                            Layout.fillWidth: true
                            SvgIcon { pathData: "<path d='M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4'/><path d='M9 18c-4.51 2-5-2-7-2'/>"; size: 20; iconColor: textMuted }
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                Label { text: qsTr("GitHub Repository"); color: textMain; font.weight: Font.Medium; Layout.fillWidth: true; elide: Text.ElideRight }
                                Label { text: qsTr("Created by sh1tpostkun"); color: textMuted; font.pixelSize: 12; Layout.fillWidth: true; elide: Text.ElideRight }
                            }
                            Button {
                                text:qsTr("Open")
                                flat: true
                                background: Rectangle { color: bgSecondary; radius: 6 }
                                contentItem: Label { text: parent.text; color: textMain; font.pixelSize: 12; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
                                onClicked: Qt.openUrlExternally("https://github.com/sh1tpostkun/acer-control")
                            }
                        }
                    }
                }
            }
            
            Item { height: 32; Layout.fillWidth: true } // bottom spacer
        }
    }
}

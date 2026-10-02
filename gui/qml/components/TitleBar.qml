import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

Rectangle {
    id: titleBar
    height: 40
    Layout.preferredHeight: 40
    Layout.minimumHeight: 40
    Layout.maximumHeight: 40
    color: "#08111D"
    
    property Window window
    property bool rightPanelVisible: false
    signal toggleRightPanel()

    MouseArea {
        anchors.fill: parent
        onPressed: if (window) window.startSystemMove()
        onDoubleClicked: {
            if (window) {
                if (window.visibility === Window.Maximized)
                    window.showNormal()
                else
                    window.showMaximized()
            }
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 16
        anchors.rightMargin: 16
        spacing: 16

        Image {
            source: "qrc:/AcerControl/res/logo.png"
            sourceSize: Qt.size(20, 20)
            Layout.alignment: Qt.AlignVCenter
        }

        Label {
            text: "AcerControl"
            color: "#E8F0FA"
            font.weight: Font.Medium
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter
        }


        Rectangle {
            width: 32
            height: 32
            radius: 6
            color: titleBar.rightPanelVisible ? "#438BFF" : "transparent"
            Layout.alignment: Qt.AlignVCenter
            Layout.rightMargin: 8
            
            SvgIcon {
                anchors.centerIn: parent
                pathData: "<rect x='3' y='3' width='18' height='18' rx='2' ry='2'/><line x1='15' y1='3' x2='15' y2='21'/>"
                size: 16
                iconColor: titleBar.rightPanelVisible ? "#FFFFFF" : "#8FA3BA"
            }
            
            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onEntered: if (window && !titleBar.rightPanelVisible) parent.color = "#192A40"
                onExited: if (window && !titleBar.rightPanelVisible) parent.color = "transparent"
                onPressed: (mouse) => mouse.accepted = true
                preventStealing: true
                onClicked: titleBar.toggleRightPanel()
            }
        }
        
        RowLayout {
            spacing: 0
            
            Repeater {
                model: [
                    { path: "<line x1='5' y1='12' x2='15' y2='12'/>", action: "minimize" },
                    { path: "<rect x='5' y='5' width='10' height='10'/>", action: "maximize" },
                    { path: "<line x1='5' y1='5' x2='15' y2='15'/><line x1='15' y1='5' x2='5' y2='15'/>", action: "close" }
                ]
                
                Rectangle {
                    width: 36
                    height: 36
                    color: "transparent"
                    Layout.alignment: Qt.AlignVCenter
                    
                    SvgIcon {
                        anchors.centerIn: parent
                        size: 20
                        pathData: modelData.path
                        iconColor: "#8FA3BA"
                    }
                    
                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        onEntered: parent.color = modelData.action === "close" ? "#E85B6A" : "#192A40"
                        onExited: parent.color = "transparent"
                        onPressed: mouse.accepted = true
                        preventStealing: true
                        onClicked: {
                            if (modelData.action === "minimize") window.showMinimized()
                            else if (modelData.action === "maximize") {
                                if (window.visibility === Window.Maximized) window.showNormal()
                                else window.showMaximized()
                            }
                            else if (modelData.action === "close") window.close()
                        }
                    }
                }
            }
        }
    }
}

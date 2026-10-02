import QtQuick

Image {
    id: root
    property string pathData: ""
    property color iconColor: "#8FA3BA"
    property int size: 20

    sourceSize.width: size
    sourceSize.height: size
    width: size
    height: size

    source: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='" + iconColor.toString().replace('#', '%23') + "' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'>" + pathData + "</svg>"
    fillMode: Image.Pad
    asynchronous: true
}

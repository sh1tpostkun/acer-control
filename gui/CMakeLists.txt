cmake_minimum_required(VERSION 3.16)
project(AcerControl VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Widgets Qml Quick QuickControls2 Network LinguistTools)

qt_add_executable(acercontrol-gui
    src/main.cpp src/BackendClient.cpp
)

qt_add_qml_module(acercontrol-gui
    URI AcerControl
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/pages/Dashboard.qml
        qml/pages/Performance.qml
        qml/pages/Fans.qml
        qml/pages/Battery.qml
        qml/pages/Monitoring.qml
        qml/pages/Keyboard.qml
        qml/pages/Settings.qml
        qml/pages/Diagnostics.qml
        qml/pages/Display.qml
        qml/pages/Network.qml
        qml/pages/Devices.qml
        qml/components/SvgIcon.qml
        qml/components/TitleBar.qml
    RESOURCES
        res/logo.png
        res/app_icon.png
)

qt_add_translations(acercontrol-gui
    TS_FILES
        i18n/en.ts
        i18n/ru.ts
    RESOURCE_PREFIX "/i18n"
)

target_link_libraries(acercontrol-gui PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Widgets
    Qt6::Qml
    Qt6::Quick
    Qt6::QuickControls2 Qt6::Network
)

install(TARGETS acercontrol-gui
    BUNDLE DESTINATION .
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)

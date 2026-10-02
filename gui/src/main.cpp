#include <QApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include "BackendClient.h"
#include <QCommandLineParser>
#include <QFile>
#include <QSettings>
#include <QIcon>
#include <QTranslator>
#include <QLocale>

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    app.setApplicationName("AcerControl");
    app.setOrganizationName("AcerControl");
    app.setWindowIcon(QIcon(":/AcerControl/res/app_icon.png"));

    QCommandLineParser parser;
    parser.setApplicationDescription("AcerControl GUI");
    parser.addHelpOption();
    
    QCommandLineOption mockOption("mock", "Use mock socket");
    parser.addOption(mockOption);
    parser.process(app);
    
    QSettings settings;
    QString lang = settings.value("language", "system").toString();
    if (lang == "system") {
        lang = (QLocale::system().language() == QLocale::Russian) ? "ru" : "en";
    }

    QTranslator *translator = new QTranslator(&app);
    bool loaded = translator->load(":/i18n/" + lang + ".qm");
    qDebug() << "Loading language:" << lang << "Success:" << loaded;
    if (loaded) {
        app.installTranslator(translator);
    }

    BackendClient backend;
    QQmlApplicationEngine engine;
    
    QObject::connect(&backend, &BackendClient::languageChanged, [&app, translator, &engine](QString newLang) {
        if (translator->load(":/i18n/" + newLang + ".qm")) {
            app.installTranslator(translator);
            engine.retranslate();
        }
    });

    QString socketPath = "/run/acercontrol/daemon.sock";
    if (parser.isSet(mockOption) || QFile::exists("/tmp/acercontrol_mock.sock")) {
        socketPath = "/tmp/acercontrol_mock.sock";
    }

    backend.connectToServer(socketPath);

    engine.rootContext()->setContextProperty("backend", &backend);

    const QUrl url(u"qrc:/AcerControl/qml/Main.qml"_qs);
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreated,
        &app, [url](QObject *obj, const QUrl &objUrl) {
            if (!obj && url == objUrl)
                QCoreApplication::exit(-1);
        }, Qt::QueuedConnection);
    engine.load(url);

    return app.exec();
}

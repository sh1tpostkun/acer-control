#ifndef BACKENDCLIENT_H
#define BACKENDCLIENT_H

#include <QObject>
#include <QLocalSocket>
#include <QJsonObject>
#include <QJsonArray>

class QTimer;

class BackendClient : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool connected READ isConnected NOTIFY connectedChanged)
    Q_PROPERTY(QJsonObject systemInfo READ systemInfo NOTIFY systemInfoChanged)
    Q_PROPERTY(QJsonObject fanStatus READ fanStatus NOTIFY fanStatusChanged)
    Q_PROPERTY(QJsonObject capabilities READ capabilities NOTIFY capabilitiesChanged)
    Q_PROPERTY(QJsonObject batteryStatus READ batteryStatus NOTIFY batteryStatusChanged)
    Q_PROPERTY(QString thermalProfile READ thermalProfile NOTIFY thermalProfileChanged)
    Q_PROPERTY(bool wifiEnabled READ wifiEnabled NOTIFY wifiEnabledChanged)
    Q_PROPERTY(bool bluetoothEnabled READ bluetoothEnabled NOTIFY bluetoothEnabledChanged)
    Q_PROPERTY(QJsonArray monitors READ monitors NOTIFY monitorsChanged)
    Q_PROPERTY(double networkDown READ networkDown NOTIFY networkSpeedChanged)
    Q_PROPERTY(double networkUp READ networkUp NOTIFY networkSpeedChanged)

public:
    explicit BackendClient(QObject *parent = nullptr);
    bool isConnected() const;

    QJsonObject systemInfo() const;
    QJsonObject fanStatus() const;
    QJsonObject capabilities() const;
    QJsonObject batteryStatus() const;
    QString thermalProfile() const;
    bool wifiEnabled() const;
    bool bluetoothEnabled() const;
    QJsonArray monitors() const;
    double networkDown() const;
    double networkUp() const;

    Q_INVOKABLE void connectToServer(const QString &socketPath);
    Q_INVOKABLE void sendRequest(const QString &type, const QJsonObject &data = QJsonObject());
    
    Q_INVOKABLE void setFanMode(const QString &mode);
    Q_INVOKABLE void setFanSpeed(int cpu, int gpu);
    Q_INVOKABLE void setThermalProfile(const QString &profile);
    Q_INVOKABLE void setBatteryLimit(int limit);
    Q_INVOKABLE void setKeyboardTimeout(int value);
    
    Q_INVOKABLE void setWifiEnabled(bool enabled);
    Q_INVOKABLE void setBluetoothEnabled(bool enabled);
    Q_INVOKABLE void dropCaches();
    
    Q_INVOKABLE void setLanguage(const QString &lang);
    Q_INVOKABLE QString getLanguage();
    
    Q_INVOKABLE void setMonitorBrightness(const QString &name, int percent);
    Q_INVOKABLE void setMonitorMode(const QString &name, const QString &modeId);
    Q_INVOKABLE void refreshMonitors();

signals:
    void toggleGuiRequested();
    void connectedChanged(bool connected);
    void systemInfoChanged(QJsonObject info);
    void fanStatusChanged(QJsonObject status);
    void capabilitiesChanged(QJsonObject caps);
    void batteryStatusChanged(QJsonObject status);
    void thermalProfileChanged(QString profile);
    void languageChanged(QString lang);
    void wifiEnabledChanged(bool enabled);
    void bluetoothEnabledChanged(bool enabled);
    void monitorsChanged(QJsonArray monitors);
    void networkSpeedChanged();
    
private slots:
    void onReadyRead();
    void onConnected();
    void onDisconnected();
    void onErrorOccurred(QLocalSocket::LocalSocketError socketError);
    void refreshNetworkState();
    void refreshNetworkSpeed();

private:
    void handleMessage(const QJsonObject &msg);
    void refreshAll();

    QLocalSocket *m_socket;
    bool m_connected;
    QJsonObject m_systemInfo;
    QJsonObject m_fanStatus;
    QJsonObject m_capabilities;
    QJsonObject m_batteryStatus;
    QString m_thermalProfile;
    bool m_wifiEnabled;
    bool m_bluetoothEnabled;
    QJsonArray m_monitors;
    double m_networkDown;
    double m_networkUp;
    unsigned long long m_lastNetRx;
    unsigned long long m_lastNetTx;
    QTimer *m_netSpeedTimer;
};

#endif // BACKENDCLIENT_H

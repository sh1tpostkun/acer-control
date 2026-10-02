void BackendClient::setFanMode(const QString &mode)
{
    QJsonObject data;
    data["mode"] = mode;
    sendRequest("SetFanMode", data);
}

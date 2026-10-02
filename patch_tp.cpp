void BackendClient::setThermalProfile(const QString &profile)
{
    QJsonObject data;
    data["profile"] = profile;
    sendRequest("SetThermalProfile", data);
}

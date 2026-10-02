void BackendClient::setBatteryLimit(int limit)
{
    QJsonObject data;
    data["limit"] = limit;
    sendRequest("SetBatteryLimit", data);
}

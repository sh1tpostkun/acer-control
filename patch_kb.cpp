void BackendClient::setKeyboardTimeout(int value)
{
    QJsonObject data;
    data["timeout_s"] = value;
    sendRequest("SetKeyboardTimeout", data);
}

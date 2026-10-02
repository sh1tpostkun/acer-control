void BackendClient::dropCaches()
{
    sendRequest("DropCaches");
}

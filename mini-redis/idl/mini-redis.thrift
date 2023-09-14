namespace rs volo.example

struct GetItemRequest {
    1: required i32 opcode,
    2: required string key_channal,
    3: required string value_message,
}

struct GetItemResponse {
    1: required i32 opcode,
    2: required string key_channal,
    3: required string value_message,
    4: required bool success,
}

service ItemService {
    GetItemResponse GetItem (1: GetItemRequest req),
}

## Http Api

Front end can send command to vessel via HTTP, search request, user info, queued download etc.
Note that there are two type of query : 
- **Soulseek commands** : those query will be forwarded to the soulseek server (or directly to a peer connection) 
  and a response will be asynchronously sent via server sent event (read the [SSE chapter](sse.md) for more info]. This type of request 
  always respond with status code [202 Accepted](https://developer.mozilla.org/fr/docs/Web/HTTP/Status/202).
- **Vessel requests** : Unlike Soulseek commands this kind of request sent a response, either cached in vessel memory or
    persisted in the embedded database.
  
#### Search

- `GET /search` : Send a search query to Soulseek. Vessel will send peer replies via `search_reply` SSE events.
  ```shell
  curl -X GET http://localhost:3030/search?term=%22Nirvana%22
  ```
  
#### Chat

- `GET /chat/start` : Ask Soulseek server to send us messages from all public rooms, also known as public chat.

    ```shell
    curl -X GET http://localhost:3030/chat/start 
    ```

- `GET /chat/stop` : Ask Soulseek server to stop sending us messages from all public rooms.

    ```shell
    curl -X GET http://localhost:3030/chat/stop 
    ```
- `GET /rooms/{room_name}/join` : We want to join a room.

    ```shell
    curl -X GET http://localhost:3030/rooms/Underground%20Hiphop/join
    ```

- `POST /rooms/{room_name}` : Send a chat message to a room.

    ```shell
    curl -X POST http://localhost:3030/rooms/Underground%20Hiphop \
    --header 'Content-Type: application/json' \
    --data '{
	    "message": "This is a test message from Vessel server, vessel is a self hostable soulseek client unde active developpement."
    }'
    ```

#### Peers
  
- `POST /peers/{peer_name}/queue` : Ask a peer to upload a file. If the download is being successfully queued vessel will 
  reply with a `download_started` event and subsequend progress will be advertised with `download_progress` events. 
    ```shell
    curl -X POST http://localhost:3030/peers/fidaRM/queue \
    --header 'Content-Type: application/json' \
    --data '{
	  "filename": "@@zsttx\\Musica\\Importati\\Nirvana\\1991 - Nevermind\\12 - Something in the Way _ Endless.flac"
    }'
    ```
  
- `GET peers/{peer_name}/shares` : Ask a peer to send is shared directories. 
    ```shell
    curl -X GET http://localhost:3030/peers/JacquesDurand123456@/shares
    ```
  
### Users

- `GET /users` : Return a list of known users stored in our local database.
    ```shell
    curl -X GET http://localhost:3030/users
    ```
    **Response**: 
    ```json
    [
      {
        "username": "60'",
        "ip": "78.28.37.115",
        "port": 50526
      },
      {
        "username": "60smonomanic",
        "ip": "94.211.42.17",
        "port": 22874
      }
    ]
    ```
  
- `GET users/{user_name}/info` : Ask Soulseek server to send user info on a user.
    ```shell
    curl -X GET http://localhost:3030/users/JacquesDurand123456@/info
    ```

- `GET users/{user_name}/status` : Ask Soulseek server to send user status for a given username.
    ```shell
    curl -X GET http://localhost:3030/users/JacquesDurand123456@/info
    ```

- `GET users/{user_name}/status` : Ask Soulseek server to send a user ip address.
    ```shell
    curl -X GET http://localhost:3030/users/JacquesDurand123456@/address
    ```
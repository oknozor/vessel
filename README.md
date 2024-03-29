## Vessel 
A self-hostable soulseek client. 

This is still a **WORK IN PROGESS** and not usable yet.
  
## Useful links

- [soulseek protocol messages](https://nicotine-plus.github.io/nicotine-plus/doc/SLSKPROTOCOL.html#server-code-1)
- [nicotine+](https://github.com/Nicotine-Plus/nicotine-plus)
- [museek+](https://github.com/eLvErDe/museek-plus)

## Running

### Network config

Make sure port 2255 is open and transfered to your machine ip, peer can attempt direct connection to your vessel instance. 

### vessel server

```shell
RUST_LOG=debug cargo run
```

#### Filtering logs

If you want to log a specific module please refer to https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html

### vessel web
```shell
cd vessel_web
npm install
npm run dev
```

## Architecture

### modules 

- [soulseek_protocol](soulseek_protocol) :
  This crate provide serializaion and deserialisation for soulssek TCP message.
  
- [vessel_sse](vessel_sse) : Dispatch message from soulseek to the web client.
    Messages are sent in json format with the following content : 
    ```json
  {
      "UserJoinedRoom": { // message type
        // content ...
        "room": "ARGENTINA",
        "username": "vessel",
        "status": 2,
        "avgspeed": 0,
        "downloadnum": 0,
        "files": 0,
        "dirs": 0,
        "slotsfree": 0,
        "countrycode": "FR"
      }
    }
    ```
  For a list of available messages run the following command : 
  ```shell
  cargo doc --workspace
  cargo doc --open
  ```
  And browse to `soulseek_protocol/server.server::messages/request/enum.ServerRequest.html`

  
- [vessel_server](vessel_server) :
  The main process : 
    - Initiate logging
    - Spawn TCP listeners (Soulseek server + outgoing/incoming distributed connections).
    - Spawn HTTP and SSE task.
    - Send login credentials to soulseek after every process has started.
  
  
- [vessel_http](vessel_http) : 

  REST api endpoints to communicate with the server, note that these endpoints are "write only".
  most of them will answer with 202/Accepted, response will be later send via server sent event.
  
- [vessel_web](vessel_web) :

  A web application, sending request via HTTP and getting response asynchronously via SSE. 

### Message flow diagram 

![Message flow diagram](docs/diagrams/architecture.png)

### How to 

### TCP trace in wireshark

```
(ip.src == 192.168.0.17 && ip.dst==208.76.170.59) && tcp.port == 2242 && tcp && tcp.analysis.push_bytes_sent
```


### Get protocol message using slsk_dump

1. Start slsk_dump

```shell
cargo build
sudo ./target/debug/slsk_dump
```

2. Run nicotine or another soulseek client to produce the desired messages
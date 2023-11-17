- [**Deployment**](#deployment)
  - [**Build Images**](#build-images)
  - [**Deployimet in GCE VM**](#deployimet-in-gce-vm)
- [**Usage**](#usage)
  - [**Reference**](#reference)


# **Mini Rust WebSocket Server**


## **Getting started**
**WebSocket Server**

- Running WebSocket Server

  ```
  cargo run
  ```

**Remote Video Stream Test Pages**

- Installing http-server

  Before run the pages, you should install http-server

  ```
  npm i http-server
  ```

- Running test pages of sender and receiver
  ```
  npx http-server -p "PORT"
  ```
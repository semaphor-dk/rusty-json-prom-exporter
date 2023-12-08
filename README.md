# Rustbased exporter that converts arbitrary JSON to the .prom format.

This small application works great in tandem with the Prometheus [node-exporter](https://github.com/prometheus/node_exporter).
If you place the converted JSON -> prom data in the /var/lib/prometheus/node-exporter/ directory, the metrics will be added to the node-exporter output.

Build the project using Cargo.

Run the binary as 

```
./rusty-json-prom-exporter [url] [filename]
```

or better yet, set up a linux systemd service:

```
[Unit]
Description=Exporter that converts arbitrary JSON to the .prom data format - written in Rust.

[Service]
ExecStart=<path to binary> <url> <filename>.prom
Restart=always
User=root

[Install]
WantedBy=multi-user.target
```

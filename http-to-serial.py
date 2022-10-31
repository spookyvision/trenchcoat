#!/usr/bin/env python3
"""
Forward HTTP POST to a serial device.

Usage::
    ./server.py <device>
"""
from http.server import BaseHTTPRequestHandler, HTTPServer
import logging


class S(BaseHTTPRequestHandler):
    def _set_response(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin:", "*")
        self.send_header("Content-type", "text/plain")
        self.end_headers()

    def do_GET(self):
        self._set_response()
        self.wfile.write(b"")

    def do_OPTIONS(self):
        self._set_response()
        self.wfile.write(b"")

    def do_POST(self):
        content_length = int(
            self.headers["Content-Length"]
        )  # <--- Gets the size of data
        post_data = self.rfile.read(content_length)  # <--- Gets the data itself
        self._set_response()
        with open(dev, "wb") as fh:
            _wrote = fh.write(post_data)
            # logging.info("read %s, wrote %s bytes" % (content_length, _wrote))
        self.wfile.write("POST request for {}".format(self.path).encode("utf-8"))


def run(server_class=HTTPServer, handler_class=S, port=8008):
    logging.basicConfig(level=logging.INFO)
    server_address = ("", port)
    httpd = server_class(server_address, handler_class)
    logging.info("Starting httpd...\n")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    httpd.server_close()
    logging.info("Stopping httpd...\n")


if __name__ == "__main__":
    from sys import argv

    dev = argv[1]
    run()

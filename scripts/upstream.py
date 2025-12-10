from http.server import HTTPServer, BaseHTTPRequestHandler
import sys

class S(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        self.wfile.write(b'{"status":"ok"}')
    
    def log_message(self, format, *args):
        return

if __name__ == '__main__':
    print("Starting upstream on :8080")
    try:
        HTTPServer(('127.0.0.1', 8080), S).serve_forever()
    except KeyboardInterrupt:
        pass

// Hello World HTTP server — handler as closure, not global

http
    .createServer(
        function handler(req) {
            console.log('hi', req.url)
            return {
                status: 200,
                headers: { "Content-Type": "text/plain" },
                body: "Hello, World!"
            }
        })
    .listen(8888)

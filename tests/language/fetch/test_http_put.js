// http.put() → PUT /update
function test() {
    function handleResponse(res) {
        return res.json();
    }
    return http.put("http://localhost:8888/update", { id: 1, title: "updated" }).then(handleResponse);
}
test()

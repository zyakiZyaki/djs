// Callback chain pattern — start/stop process
function test() {
    function process({OnStart, OnStop}) {
        return function() {
            return OnStart(function() { OnStop() });
        };
    }
    function runTask(cb) {
        return cb();
    }
    function cleanup() {
        return "done";
    }
    return process({OnStart: runTask, OnStop: cleanup})();
}
test()

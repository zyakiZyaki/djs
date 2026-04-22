// ERROR: calling global function
function dirty() {
    return otherFunc();
}
dirty()

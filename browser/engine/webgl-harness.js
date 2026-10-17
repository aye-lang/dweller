(function () {
  var passed = 0;

  window.webglTestHarness = {
    reportResults: function (path, success, msg, skipped) {
      if (skipped) return;
      if (success) {
        passed++;
        return;
      }
      console.log("DWELLER_WEBGL fail " + String(msg).replace(/\s+/g, " "));
    },
    notifyFinished: function () {
      console.log("DWELLER_WEBGL done " + passed);
    }
  };
})();

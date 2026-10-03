angular.module('beamng.apps').directive('avatarSandboxControls', ['$interval', function ($interval) {
  return {
    restrict: 'EA', replace: true,
    templateUrl: '/ui/modules/apps/avatarSandboxControls/panel.html',
    link: function (scope) {
      var allowed = ['refresh','toggleMod','roblox','beamng','building','interact','clearTelemetry'];
      ['forward','backward','left','right','jump'].forEach(function (key) { allowed.push(key+'0',key+'1'); });
      scope.state = {mod:{enabled:false,characterMode:'beamng'},driver:{attached:false},building:{enabled:false},telemetry:{errorCount:0,events:[]}};
      scope.report = '';
      scope.send = function (command) {
        if (allowed.indexOf(command) < 0) return;
        bngApi.engineLua("if avatarSandbox_controls then avatarSandbox_controls.command('"+command+"') end");
      };
      var held = {};
      scope.press = function (key) { held[key]=true; scope.send(key+'1'); };
      scope.release = function () {
        Object.keys(held).forEach(function (key) { scope.send(key+'0'); }); held={};
      };
      window.addEventListener('mouseup',scope.release);
      window.addEventListener('blur',scope.release);
      scope.$on('AvatarSandboxStatus',function (event,data) { scope.$evalAsync(function () {scope.state=data;}); });
      scope.makeReport = function () { scope.report=JSON.stringify(scope.state,null,2); };
      bngApi.engineLua("extensions.load('avatarSandbox_controls')");
      var timer=$interval(function () {scope.send('refresh');},1000);
      scope.$on('$destroy',function () {
        scope.release(); $interval.cancel(timer);
        window.removeEventListener('mouseup',scope.release);window.removeEventListener('blur',scope.release);
      });
    }
  };
}]);

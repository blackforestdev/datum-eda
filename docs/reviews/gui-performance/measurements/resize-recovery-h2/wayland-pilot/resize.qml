import QtQuick
import org.kde.kwin
Item {
 property var owned: null
 property int count: 0
 property rect original
 Component.onCompleted: {
  let matches = Workspace.windowList().filter(w => w.pid === 173390);
  if (matches.length !== 1) { console.error("DATUM pilot refuses ambiguous target"); return; }
  owned = matches[0]; original = owned.frameGeometry; timer.start();
 }
 Timer { id: timer; interval: 20; repeat: true
  onTriggered: {
   if (!owned || owned.pid !== 173390) { stop(); return; }
   if (count >= 240) { owned.frameGeometry = original; stop(); return; }
   let delta = Math.abs((count % 120)-60)*3;
   owned.frameGeometry = Qt.rect(original.x, original.y, 1200+delta, 780);
   count++;
  }
 }
}

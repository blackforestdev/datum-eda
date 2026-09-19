import QtQuick
import org.kde.kwin
Item {
 property var owned: null
 property int count: 0
 property rect original
 function pairValid() {
  let windows = Workspace.stackingOrder.filter(w => w.pid === 344319);
  return windows.length === 2 && windows.filter(w => w.caption.includes("New Project") && !w.minimized).length === 1;
 }
 Component.onCompleted: {
  if (!pairValid()) { console.error("DATUM requested dialog absent"); return; }
  let matches = Workspace.stackingOrder.filter(w => w.pid === 344319 && !w.caption.includes("Preferences") && !w.caption.includes("New Project"));
  if (matches.length !== 1) { console.error("DATUM refuses ambiguous resize target"); return; }
  owned = matches[0]; original = owned.frameGeometry; timer.start();
 }
 Timer { id: timer; interval: 20; repeat: true
  onTriggered: {
   if (!owned || owned.pid !== 344319 || !pairValid()) { stop(); return; }
   if (count === 240) {
    owned.frameGeometry = Qt.rect(original.x, original.y, original.width+17, original.height+17);
    count++; return;
   }
   if (count > 240 && count < 245) { count++; return; }
   if (count >= 245) { owned.frameGeometry = original; stop(); return; }
   let delta = Math.abs((count % 120)-60)*3;
   owned.frameGeometry = Qt.rect(original.x, original.y,
      original.width + ("height" === "width" ? delta : 0),
      original.height + ("height" === "height" ? delta : 0));
   count++;
  }
 }
}

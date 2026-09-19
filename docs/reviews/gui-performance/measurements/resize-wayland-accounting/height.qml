import QtQuick
import org.kde.kwin
Item {
 property var owned: null
 property int count: 0
 property rect original
 Component.onCompleted: {
  let matches = Workspace.stackingOrder.filter(w => w.pid === 340685);
  if (matches.length !== 1) { console.error("DATUM refuses ambiguous resize target"); return; }
  owned = matches[0]; original = owned.frameGeometry; timer.start();
 }
 Timer { id: timer; interval: 20; repeat: true
  onTriggered: {
   if (!owned || owned.pid !== 340685) { stop(); return; }
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

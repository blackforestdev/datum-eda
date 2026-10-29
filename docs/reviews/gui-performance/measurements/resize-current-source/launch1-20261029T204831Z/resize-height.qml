import QtQuick
import org.kde.kwin
Item {
 property var owned: null
 property int count: 0
 property rect original
 Component.onCompleted: {
  let matches = Workspace.stackingOrder.filter(w => w.pid === 3932011);
  if (matches.length !== 1) { console.error("DATUM refuses ambiguous resize target"); return; }
  owned = matches[0]; original = owned.frameGeometry; timer.start();
 }
 Timer { id: timer; interval: 16; repeat: true
  onTriggered: {
   if (!owned || owned.pid !== 3932011) { stop(); return; }
   if (count >= 600) { owned.frameGeometry = original; stop(); return; }
   let delta = Math.abs((count % 120) - (120/2)) * (180.0/(120/2));
   owned.frameGeometry = Qt.rect(original.x, original.y,
      original.width  + ("height" === "width"  ? delta : 0),
      original.height + ("height" === "height" ? delta : 0));
   count++;
  }
 }
}

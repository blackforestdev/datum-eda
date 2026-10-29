import QtQuick
import org.kde.kwin
Item {
 Component.onCompleted: {
  let m = Workspace.stackingOrder.filter(w => w.pid === 3932011);
  if (m.length !== 1) { console.error("DATUM refuses ambiguous resize target"); return; }
  let o = m[0]; let g = o.frameGeometry;
  o.frameGeometry = Qt.rect(g.x, g.y,
     g.width  + ("width" === "width"  ? 180 : 0),
     g.height + ("width" === "height" ? 180 : 0));
 }
}

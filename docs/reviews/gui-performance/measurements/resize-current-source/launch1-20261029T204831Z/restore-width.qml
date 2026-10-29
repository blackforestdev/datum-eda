import QtQuick
import org.kde.kwin
Item { Component.onCompleted: {
  let m = 1280orkspace.stackingOrder.filter(w => w.pid === 3932011);
  if (m.length === 1) { let o=m[0]; let g=o.frameGeometry; o.frameGeometry = Qt.rect(g.x,g.y,1280,800); }
}}
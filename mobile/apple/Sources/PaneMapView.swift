// ABOUTME: Draws the selected Herdr tab's split layout from reported pane bounds.
// ABOUTME: Sends the tapped pane ID through the session's ordinary focus action.
import SwiftUI

struct PaneMapView: View {
    let view: MobileView
    let select: (String) -> Void

    var body: some View {
        if let map = view.paneMap {
            let panes = Dictionary(view.panes.filter { $0.tabId == view.focusedTab }.map { ($0.id, $0) },
                                   uniquingKeysWith: { first, _ in first })
            GeometryReader { proxy in
                let dx = proxy.size.width / CGFloat(map.columns)
                let dy = proxy.size.height / CGFloat(map.rows)
                ZStack(alignment: .topLeading) {
                    ForEach(map.panes, id: \.id) { rect in
                        if let pane = panes[rect.id] {
                            let width = CGFloat(rect.width) * dx
                            let height = CGFloat(rect.height) * dy
                            tile(pane, size: CGSize(width: width, height: height))
                                .offset(x: CGFloat(rect.x) * dx, y: CGFloat(rect.y) * dy)
                        }
                    }
                }
                .frame(width: proxy.size.width, height: proxy.size.height, alignment: .topLeading)
            }
            .aspectRatio(1.8, contentMode: .fit)
            .frame(maxHeight: 240)
            .accessibilityElement(children: .contain)
            .accessibilityLabel("Pane map")
        }
    }

    private func tile(_ pane: MobilePane, size: CGSize) -> some View {
        let selected = pane.id == view.focusedPane
        let activity = PaneActivity(pane.status)
        let shape = RoundedRectangle(cornerRadius: 18)
        return Button { select(pane.id) } label: {
            ZStack {
                shape.fill(Color(uiColor: .secondarySystemGroupedBackground))
                if size.height >= 24 && size.width >= 32 {
                    VStack(alignment: .leading, spacing: 4) {
                        HStack(spacing: 5) {
                            if let activity { activity.dot }
                            Text(verbatim: pane.label).font(.subheadline.weight(.semibold)).lineLimit(1)
                        }
                        if size.height >= 55, let activity { Text(activity.title).font(.caption).lineLimit(1) }
                    }
                    .foregroundStyle(.primary)
                    .padding(8)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                }
                shape.strokeBorder(selected ? (activity == .blocked ? Color.accentColor : .white) : .secondary.opacity(0.4), lineWidth: selected ? 2 : 1)
            }
            .padding(1)
            .frame(width: size.width, height: size.height)
            .clipped()
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(Text(verbatim: pane.label))
        .accessibilityValue(activity?.title ?? "")
        .accessibilityAddTraits(selected ? .isSelected : [])
        .accessibilityIdentifier("pane.map.\(pane.id)")
    }
}

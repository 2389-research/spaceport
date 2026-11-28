// ABOUTME: Measures native row creation in the collapsed rail with a long idle group.
// ABOUTME: Exercises the same scrolling layout used for the agent buttons.
import SwiftUI
import XCTest
@testable import SpaceportMobile

@MainActor
final class AgentRailLayoutTests: XCTestCase {
    private struct Probe: UIViewRepresentable {
        let created: () -> Void

        func makeUIView(context: Context) -> UIView {
            created()
            return UIView()
        }

        func updateUIView(_ view: UIView, context: Context) {}
    }

    func testLongIdleGroupBuildsRowsAsTheyScrollIntoView() throws {
        let workspace = MobileWorkspace(id: "w", label: "Project", status: "idle")
        let tab = MobileTab(id: "t", workspaceId: workspace.id, label: "Tab", status: "idle")
        let agents = (0..<37).map { index in
            GoToPane(workspace: workspace, tab: tab, pane: MobilePane(stateChangeSeq: nil, agentName: "Scout",
                      id: String(index), tabId: tab.id, label: "Pane", status: "idle"))
        }
        let rail = AgentRail(sections: AgentRail.groups(agents), focused: nil, expand: {}, select: { _ in })
        var mounted = Set<String>()
        var scroll: ScrollViewProxy?
        let host = UIHostingController(rootView: ScrollViewReader { proxy in
            rail.rows { row in
                Probe { mounted.insert(row.id) }.frame(width: 44, height: 44)
            }
            .frame(width: 56, height: 220)
            .onAppear { scroll = proxy }
        })
        let scene = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first)
        let window = UIWindow(windowScene: scene)
        window.rootViewController = host
        window.makeKeyAndVisible()
        defer { window.isHidden = true }
        host.view.layoutIfNeeded()
        let last = try XCTUnwrap(agents.last).id
        XCTAssertTrue(mounted.contains(try XCTUnwrap(agents.first).id))
        XCTContext.runActivity(named: "Initially mounted \(mounted.count) of \(agents.count) rows") { _ in }
        XCTAssertFalse(mounted.contains(last), "Initially built \(mounted.count) of \(agents.count) rows")
        try XCTUnwrap(scroll).scrollTo(last, anchor: .bottom)
        host.view.setNeedsLayout()
        host.view.layoutIfNeeded()
        XCTAssertTrue(mounted.contains(last), "Scrolling should materialize the destination")
    }
}

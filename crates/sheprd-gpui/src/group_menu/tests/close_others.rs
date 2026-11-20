use {super::*, crate::window::MockPeer, sheprd_client::protocol::AgentStatus};

// These sequence tests drive their own projection; rendering the terminal
// would resize it and make that projection unready between confirmations.
struct Fixture(Entity<SheprdWindow>);

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// A split whose left group lists four Herdr tabs, each with an agent at
/// work so its close asks first, and shows "t0".
fn busy_split(
    cx: &mut TestAppContext,
) -> (
    Entity<SheprdWindow>,
    &mut VisualTestContext,
    GroupId,
    MockPeer,
) {
    let peer = MockPeer::advertising(&["tab.close"]);
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            peer.prepare(&mut view, cx);
            let mut shown = snapshot(40);
            // Keep the real worker's acknowledged boot when expanding its fixture projection.
            shown.boot_id = view.live.snapshot.as_ref().unwrap().boot_id.clone();
            shown.focused_workspace_id = Some("w0".into());
            shown.focused_tab_id = Some("t0".into());
            let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
            surface.boot_id = shown.boot_id.clone();
            surface.projection_revision = shown.revision;
            view.live.snapshot = Some(Arc::new(shown));
            view
        }))
    });
    let view = fixture.read_with(cx, |fixture, _| fixture.0.clone());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            for id in ["x", "y"] {
                let mut tab = snapshot.tabs[0].clone();
                tab.tab_id = id.into();
                tab.workspace_id = "w0".into();
                snapshot.tabs.push(tab);
            }
            for tab in &mut snapshot.tabs {
                tab.agent_status = AgentStatus::Working;
            }
            *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
            view.command(Command::SplitEditor, window, cx);
        })
    });
    let left = groups(&view, cx)[0];
    (view, cx, left, peer)
}

fn herdr(ids: &[&str]) -> Vec<Pick> {
    ids.iter().map(|id| Pick::Herdr((*id).into())).collect()
}

fn tick(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| view.update(cx, |view, cx| view.advance_close_run(window, cx)));
    draw(cx);
}

fn decline(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));
    tick(view, cx);
}

#[gpui::test]
fn close_others_confirms_all_pages_then_keeps_the_tab_shown(cx: &mut TestAppContext) {
    let (view, cx) = window(cx);
    open_pages(&view, cx, 3);
    let pages = cx.update(|_, cx| view.read(cx).browser_tab_ids(cx));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.command(Command::SplitEditor, window, cx)
        })
    });
    let [left, right] = groups(&view, cx)[..] else {
        panic!("two groups")
    };
    let herdr_before = tabs(&view, cx, left).len() - pages.len();
    for page in &pages {
        move_page(&view, cx, right, *page);
    }
    assert!(actions(&view, cx, right).contains(&Action::CloseOthers));
    run(&view, cx, right, Action::CloseOthers);
    assert_eq!(tabs(&view, cx, right).len(), pages.len());
    assert_eq!(
        view.read_with(cx, |view, _| view.menu.page),
        Some(Page::ConfirmClose)
    );
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.send_close(window, cx);
        })
    });
    // Consent applies only to the group's captured other tabs.
    assert_eq!(tabs(&view, cx, right), [Pick::Page(pages[2])]);
    assert_eq!(tabs(&view, cx, left).len(), herdr_before);
    view.read_with(cx, |view, _| {
        assert!(view.menu.page.is_none());
        assert!(view.browser.closing.is_none());
    });
    // With one tab, there are no others to close.
    assert!(!actions(&view, cx, right).contains(&Action::CloseOthers));
}

#[gpui::test]
fn close_others_accepts_the_batch_once_before_the_first_request(cx: &mut TestAppContext) {
    let (view, cx, left, mut peer) = busy_split(cx);
    assert_eq!(tabs(&view, cx, left), herdr(&["t0", "t1", "x", "y"]));
    run(&view, cx, left, Action::CloseOthers);
    assert_eq!(
        view.read_with(cx, |view, _| view.menu.page),
        Some(Page::ConfirmClose)
    );
    tick(&view, cx);
    assert_eq!(tabs(&view, cx, left), herdr(&["t0", "t1", "x", "y"]));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.send_close(window, cx);
            assert!(
                view.menu.page.is_none(),
                "batch refused: {:?}; input_ready={}",
                view.menu.close,
                view.input_ready()
            );
            assert!(
                view.browser.closing.is_some(),
                "batch stopped: {:?}",
                view.notices.text()
            );
        })
    });
    let request = peer.request();
    assert_eq!(request["method"], "tab.close");
    assert_eq!(request["params"]["tab_id"], "t1");
    assert!(view.read_with(cx, |view, _| view.menu.page.is_none()
        && view.browser.closing.is_some()));
}

#[gpui::test]
fn declining_the_batch_leaves_every_tab_open(cx: &mut TestAppContext) {
    let (view, cx, left, _peer) = busy_split(cx);
    run(&view, cx, left, Action::CloseOthers);
    decline(&view, cx);
    // Nothing more is asked or closed.
    assert!(view.read_with(cx, |view, _| view.menu.page.is_none()));
    tick(&view, cx);
    assert!(view.read_with(cx, |view, _| view.menu.page.is_none()));
    assert_eq!(tabs(&view, cx, left), herdr(&["t0", "t1", "x", "y"]));
    view.read_with(cx, |view, _| assert!(view.browser.closing.is_none()));
}

// ABOUTME: Owns the theme picker's disposable native search field and subscription.
// ABOUTME: Refuses events from closed or replaced fields while write state survives.
use super::*;

pub(super) struct Input {
    pub(super) search: Entity<SearchInput>,
    _subscription: Subscription,
}

impl Input {
    pub(super) fn new(cx: &mut Context<SheprdWindow>) -> Self {
        let search = cx.new(SearchInput::new);
        search.update(cx, |input, cx| {
            input.set_placeholder_with(|| fl!("theme-picker-search"), cx)
        });
        let subscription = cx.subscribe(
            &search,
            |this, search, _: &crate::search_input::Changed, cx| {
                if this.menu.page != Some(Page::Themes) {
                    return;
                }
                let Some(picker) = &mut this.menu.themes else {
                    return;
                };
                if picker
                    .search()
                    .is_none_or(|owner| owner.entity_id() != search.entity_id())
                    || picker.accepting
                    || picker.query == search.read(cx).text()
                {
                    return;
                }
                picker.filter(search.read(cx).text(), &this.config.favorite_themes);
                this.preview_picker_selection(cx);
            },
        );
        Self {
            search,
            _subscription: subscription,
        }
    }
}

impl ThemePicker {
    pub(crate) fn search(&self) -> Option<&Entity<SearchInput>> {
        self.input.as_ref().map(|input| &input.search)
    }

    pub(crate) fn release_input(&mut self) {
        self.input = None;
    }

    pub(super) fn set_search_appearance(
        &self,
        font: &crate::config::FontConfig,
        theme: &Theme,
        cx: &mut App,
    ) {
        if let Some(search) = self.search() {
            search.update(cx, |input, cx| {
                input.set_appearance(font.clone(), theme.clone(), cx)
            });
        }
    }
}

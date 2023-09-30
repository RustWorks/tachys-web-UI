use super::attribute::Attribute;
use crate::{
    renderer::{dom::Dom, Renderer},
    view::ToTemplate,
};
use leptos_reactive::{create_render_effect, Effect};
use web_sys::{DomTokenList, Element};

#[inline(always)]
pub fn class<R>(c: impl IntoClass<R>) -> impl Attribute<R> {
    Class(c)
}

struct Class<C, R>(C)
where
    C: IntoClass<R>;

impl<C, R> Attribute<R> for Class<C, R>
where
    C: IntoClass<R>,
    R: Renderer,
{
    type State = C::State;

    fn to_html(
        &mut self,
        _buf: &mut String,
        class: &mut String,
        _style: &mut String,
    ) {
        class.push(' ');
        self.0.to_html(class);
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &Element) -> Self::State {
        self.0.hydrate::<FROM_SERVER>(el)
    }

    fn build(self, el: &Element) -> Self::State {
        self.0.build(el)
    }

    fn rebuild(self, state: &mut Self::State) {
        self.0.rebuild(state)
    }
}

impl<C, R> ToTemplate for Class<C, R>
where
    C: IntoClass<R>,
    R: Renderer,
{
    fn to_template(buf: &mut String, position: &mut crate::view::Position) {
        todo!()
    }
}

pub trait IntoClass<R: Renderer> {
    type State;

    fn to_html(&self, class: &mut String);

    fn hydrate<const FROM_SERVER: bool>(self, el: &R::Element) -> Self::State;

    fn build(self, el: &R::Element) -> Self::State;

    fn rebuild(self, state: &mut Self::State);
}

impl<'a, R> IntoClass<R> for &'a str
where
    R: Renderer,
{
    type State = (Element, &'a str);

    fn to_html(&self, class: &mut String) {
        class.push_str(self);
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &Element) -> Self::State {
        (el.to_owned(), self)
    }

    fn build(self, el: &Element) -> Self::State {
        Dom::set_attribute(el, "class", self);
        (el.to_owned(), self)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (el, prev) = state;
        if self != *prev {
            Dom::set_attribute(el, "class", self);
        }
        *prev = self;
    }
}

impl<R> IntoClass<R> for String
where
    R: Renderer,
{
    type State = (Element, String);

    fn to_html(&self, class: &mut String) {
        IntoClass::to_html(self, class);
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &R::Element) -> Self::State {
        (el.to_owned(), self)
    }

    fn build(self, el: &R::Element) -> Self::State {
        el.set_attribute("class", &self);
        (el.to_owned(), self)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (el, prev) = state;
        if &self != &*prev {
            el.set_attribute("class", &self);
        }
        *prev = self;
    }
}

impl IntoClass for (&'static str, bool) {
    type State = (DomTokenList, bool);

    fn to_html(&self, class: &mut String) {
        let (name, include) = self;
        if *include {
            class.push_str(name);
        }
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &Element) -> Self::State {
        let class_list = el.class_list();
        (class_list, self.1)
    }

    fn build(self, el: &Element) -> Self::State {
        let (name, include) = self;
        let class_list = el.class_list();
        if include {
            class_list.add_1(name);
        }
        (class_list, self.1)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (name, include) = self;
        let (class_list, prev_include) = state;
        if include != *prev_include {
            if include {
                class_list.add_1(name);
            } else {
                class_list.remove_1(name);
            }
        }
        *prev_include = include;
    }
}

impl<F, C> IntoClass for F
where
    F: Fn() -> C + 'static,
    C: IntoClass + 'static,
{
    type State = Effect<C::State>;

    fn to_html(&self, class: &mut String) {
        let mut value = self();
        value.to_html(class);
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &Element) -> Self::State {
        // TODO FROM_SERVER vs template
        let el = el.to_owned();
        create_render_effect(move |prev| {
            let value = self();
            if let Some(mut state) = prev {
                value.rebuild(&mut state);
                state
            } else {
                value.hydrate::<FROM_SERVER>(&el)
            }
        })
    }

    fn build(self, el: &Element) -> Self::State {
        let el = el.to_owned();
        create_render_effect(move |prev| {
            let value = self();
            if let Some(mut state) = prev {
                value.rebuild(&mut state);
                state
            } else {
                value.build(&el)
            }
        })
    }

    fn rebuild(self, state: &mut Self::State) {}
}

impl<F> IntoClass for (&'static str, F)
where
    F: Fn() -> bool + 'static,
{
    type State = Effect<bool>;

    fn to_html(&self, class: &mut String) {
        let (name, f) = self;
        let include = f();
        if include {
            name.to_html(class);
        }
    }

    fn hydrate<const FROM_SERVER: bool>(self, el: &Element) -> Self::State {
        // TODO FROM_SERVER vs template
        let (name, f) = self;
        let class_list = el.class_list();
        create_render_effect(move |prev| {
            let include = f();
            if Some(include) != prev {
                if include {
                    class_list.add_1(name);
                } else {
                    class_list.remove_1(name);
                }
            }
            include
        })
    }

    fn build(self, el: &Element) -> Self::State {
        let (name, f) = self;
        let class_list = el.class_list();
        create_render_effect(move |prev| {
            let include = f();
            if Some(include) != prev {
                if include {
                    class_list.add_1(name);
                } else {
                    class_list.remove_1(name);
                }
            }
            include
        })
    }

    fn rebuild(self, state: &mut Self::State) {}
}

#[cfg(test)]
mod tests {
    use crate::{
        html::{
            class::class,
            element::{p, HtmlElement},
        },
        renderer::dom::Dom,
        view::{Position, PositionState, RenderHtml},
    };

    #[test]
    fn adds_simple_class() {
        let mut html = String::new();
        let mut el: HtmlElement<_, _, _, Dom> = p(class("foo bar"), ());
        el.to_html(&mut html, &PositionState::new(Position::FirstChild));

        assert_eq!(html, r#"<p class="foo bar"></p>"#);
    }

    #[test]
    fn adds_class_with_dynamic() {
        let mut html = String::new();
        let mut el: HtmlElement<_, _, _, Dom> =
            p((class("foo bar"), class(("baz", true))), ());
        el.to_html(&mut html, &PositionState::new(Position::FirstChild));

        assert_eq!(html, r#"<p class="foo bar baz"></p>"#);
    }

    #[test]
    fn adds_class_with_dynamic_and_function() {
        let mut html = String::new();
        let mut el: HtmlElement<_, _, _, Dom> = p(
            (
                class("foo bar"),
                class(("baz", || true)),
                class(("boo", false)),
            ),
            (),
        );
        el.to_html(&mut html, &PositionState::new(Position::FirstChild));

        assert_eq!(html, r#"<p class="foo bar baz"></p>"#);
    }
}

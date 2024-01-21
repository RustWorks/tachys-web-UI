use crate::{
    route::{Method, RouteDefinition},
    router::Router,
    static_render::{StaticDataMap, StaticMode},
    SsrMode,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
};
use tachydom::{
    html::{attribute::Attribute, element::HtmlElement},
    renderer::Renderer,
    view::{Render, RenderHtml},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// A route that this application can serve.
pub struct RouteListing {
    path: String,
    leptos_path: String,
    mode: SsrMode,
    methods: HashSet<Method>,
    static_mode: Option<StaticMode>,
}

impl RouteListing {
    /// Create a route listing from its parts.
    pub fn new(
        path: impl ToString,
        leptos_path: impl ToString,
        mode: SsrMode,
        methods: impl IntoIterator<Item = Method>,
        static_mode: Option<StaticMode>,
    ) -> Self {
        Self {
            path: path.to_string(),
            leptos_path: leptos_path.to_string(),
            mode,
            methods: methods.into_iter().collect(),
            static_mode,
        }
    }

    /// The path this route handles.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The leptos-formatted path this route handles.
    pub fn leptos_path(&self) -> &str {
        &self.leptos_path
    }

    /// The rendering mode for this path.
    pub fn mode(&self) -> SsrMode {
        self.mode
    }

    /// The HTTP request methods this path can handle.
    pub fn methods(&self) -> impl Iterator<Item = Method> + '_ {
        self.methods.iter().copied()
    }

    /// Whether this route is statically rendered.
    #[inline(always)]
    pub fn static_mode(&self) -> Option<StaticMode> {
        self.static_mode
    }

    /*
    /// Build a route statically, will return `Ok(true)` on success or `Ok(false)` when the route
    /// is not marked as statically rendered. All route parameters to use when resolving all paths
    /// to render should be passed in the `params` argument.
    pub async fn build_static<IV>(
        &self,
        options: &LeptosOptions,
        app_fn: impl Fn() -> IV + Send + 'static + Clone,
        additional_context: impl Fn() + Send + 'static + Clone,
        params: &StaticParamsMap,
    ) -> Result<bool, std::io::Error>
    where
        IV: IntoView + 'static,
    {
        match self.static_mode {
            None => Ok(false),
            Some(_) => {
                let mut path = StaticPath::new(&self.leptos_path);
                path.add_params(params);
                for path in path.into_paths() {
                    path.write(
                        options,
                        app_fn.clone(),
                        additional_context.clone(),
                    )
                    .await?;
                }
                Ok(true)
            }
        }
    }*/
}

#[derive(Debug, Default)]
pub struct RouteList(Vec<(RouteListing, StaticDataMap)>);

impl RouteList {
    pub fn new(
        routes: impl IntoIterator<Item = (RouteListing, StaticDataMap)>,
    ) -> Self {
        Self(routes.into_iter().collect())
    }

    pub fn into_inner(self) -> Vec<(RouteListing, StaticDataMap)> {
        self.0
    }
}

impl RouteList {
    // this is used to indicate to the Router that we are generating
    // a RouteList for server path generation
    thread_local! {
        static IS_GENERATING: Cell<bool> = Cell::new(false);
        static GENERATED: RefCell<Option<RouteList>> = RefCell::new(None);
    }

    pub fn generate<T, Rndr>(app: impl FnOnce() -> T) -> Option<Self>
    where
        T: RenderHtml<Rndr>,
        Rndr: Renderer,
        Rndr::Node: Clone,
        Rndr::Element: Clone,
    {
        Self::IS_GENERATING.set(true);
        // run the app once, but throw away the HTML
        // the router won't actually route, but will fill the listing
        _ = app().to_html();
        Self::IS_GENERATING.set(false);
        Self::GENERATED.take()
    }

    pub fn is_generating() -> bool {
        Self::IS_GENERATING.get()
    }

    pub fn register(routes: RouteList) {
        Self::GENERATED.with(|inner| {
            *inner.borrow_mut() = Some(routes);
        });
    }
}

pub(crate) trait AddsToRouteList {
    fn add_to_route_list(&self, route_list: &mut RouteList);
}

impl<Rndr, Pat, ViewFn, Children> AddsToRouteList
    for RouteDefinition<Rndr, Pat, ViewFn, Children>
{
    fn add_to_route_list(&self, route_list: &mut RouteList) {
        println!("add to route list");
    }
}

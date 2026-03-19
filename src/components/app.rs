use yew::prelude::*;
use crate::music_theory::Key;
use crate::components::circle_view::CircleView;

#[function_component(App)]
pub fn app() -> Html {
    let selected_key = use_state(|| None::<Key>);

    let on_segment_click = {
        let selected_key = selected_key.clone();
        Callback::from(move |key: Key| {
            if *selected_key == Some(key) {
                selected_key.set(None);
            } else {
                selected_key.set(Some(key));
            }
        })
    };

    html! {
        <div class="app">
            <CircleView
                selected_key={*selected_key}
                on_segment_click={on_segment_click}
            />
        </div>
    }
}

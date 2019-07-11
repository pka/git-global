//! The `ui` subcommand: runs console UI.

use config::Config;
use cursive::align::HAlign;
use cursive::traits::*;
use cursive::view::Scrollable;
use cursive::views::{Dialog, LinearLayout, TextView};
use cursive::Cursive;
use cursive_table_view::{TableView, TableViewItem};
use errors::Result;
use repo::Repo;
use report::Report;
use std::cmp::Ordering;

const THEME: &str = r##"
shadow = false
borders = "simple"

[colors]
    background = "black"
    view       = "black"
    primary   = "#5CACEE"   # steelblue
    secondary = "#ab9df2"
    tertiary  = "#fc9867"

    title_primary   = "#FFA500"  # orange
    title_secondary = "#FF00FF"  # fuchsia

    highlight          = "#54ffff"  # aqua
    highlight_inactive = "#1E90FF"  # dodgerblue
"##;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Path,
    State,
    LastCommit,
}

impl BasicColumn {
    fn as_str(&self) -> &str {
        match *self {
            BasicColumn::Path => "Directory",
            BasicColumn::State => "State",
            BasicColumn::LastCommit => "Last commit",
        }
    }
}

#[derive(Clone)]
struct Row {
    repo: Repo,
    name: String,
    state: String,
    last_commit: String,
}

impl TableViewItem<BasicColumn> for Row {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Path => self.name.to_string(),
            BasicColumn::State => self.state.to_string(),
            BasicColumn::LastCommit => self.last_commit.to_string(),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Path => self.name.cmp(&other.name),
            BasicColumn::State => self.state.cmp(&other.state),
            BasicColumn::LastCommit => self.last_commit.cmp(&other.last_commit),
        }
    }
}

pub fn execute(mut config: Config) -> Result<Report> {
    let repos = config.get_repos();
    let report = Report::new(&repos);

    let mut siv = Cursive::default();

    siv.load_toml(THEME).unwrap();

    // We can quit by pressing q
    siv.add_global_callback('q', |s| s.quit());

    // -- table
    let mut table = TableView::<Row, BasicColumn>::new()
        .column(BasicColumn::Path, "Directory", |c| c.width_percent(60))
        .column(BasicColumn::State, "State", |c| c.align(HAlign::Center))
        .column(BasicColumn::LastCommit, "Last Commit", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        });

    let rows = repos
        .iter()
        .map(|repo| Row {
            repo: repo.clone(),
            name: repo.path().to_string(),
            state: ".".to_string(), // repo.get_short_status(),
            last_commit: format!("{}", repo.num_hours_since_last_commit()),
        })
        .collect();
    table.set_items(rows);

    table.set_on_sort(
        |siv: &mut Cursive, column: BasicColumn, order: Ordering| {
            siv.add_layer(
                Dialog::around(TextView::new(format!(
                    "{} / {:?}",
                    column.as_str(),
                    order
                )))
                .title("Sorted by")
                .button("Close", |s| {
                    s.pop_layer();
                }),
            );
        },
    );

    table.set_on_submit(|siv: &mut Cursive, _row: usize, index: usize| {
        let lines = siv
            .call_on_id(
                "table",
                move |table: &mut TableView<Row, BasicColumn>| {
                    let row = table.borrow_item(index).unwrap();
                    row.repo.get_status()
                },
            )
            .unwrap();
        siv.call_on_id("output", |v: &mut TextView| {
            v.set_content("Status\n");
            lines
                .iter()
                .for_each(|line| v.append(format!("{}\n", line)))
        });
    });

    siv.add_fullscreen_layer(
        LinearLayout::vertical()
            .child(
                Dialog::around(
                    table.with_id("table").full_width().full_height(),
                )
                .title("Repositories"),
            )
            .child(
                LinearLayout::horizontal()
                    .child(TextView::new(" Commands:").h_align(HAlign::Left))
                    .child(
                        TextView::new("q: Quit ")
                            .h_align(HAlign::Right)
                            .full_width(),
                    ),
            )
            .child(
                Dialog::around(
                    TextView::empty()
                        .with_id("output")
                        .scrollable()
                        .min_height(8),
                )
                .title("Output"),
            ),
    );

    siv.run();
    Ok(report)
}

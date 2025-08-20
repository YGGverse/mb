use anyhow::Result;
use chrono::{DateTime, Utc};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

const POST: TableDefinition<i64, String> = TableDefinition::new("post");

pub struct Post {
    pub id: i64,
    pub message: String,
}

impl Post {
    pub fn time(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_micros(self.id).unwrap()
    }
}

pub struct Posts {
    pub posts: Vec<Post>,
    pub total: usize,
}

#[derive(Clone, Debug, Default)]
pub enum Sort {
    #[default]
    Time,
}

#[derive(Clone, Debug, Default)]
pub enum Order {
    // Asc,
    #[default]
    Desc,
}

pub struct Db(Database);

impl Db {
    pub fn init(path: &str) -> Result<Self> {
        let db = Database::create(path)?;
        let tx = db.begin_write()?;
        {
            tx.open_table(POST)?; // init table
        }
        tx.commit()?;
        Ok(Self(db))
    }

    pub fn delete(&self, id: i64) -> Result<bool> {
        let tx = self.0.begin_write()?;
        let is_deleted = {
            let mut t = tx.open_table(POST)?;
            t.remove(id)?.is_some()
        };
        tx.commit()?;
        Ok(is_deleted)
    }

    pub fn submit(&self, message: String) -> Result<()> {
        let tx = self.0.begin_write()?;
        {
            let mut t = tx.open_table(POST)?;
            t.insert(Utc::now().timestamp_micros(), message)?;
        }
        Ok(tx.commit()?)
    }

    pub fn post(&self, id: i64) -> Result<Option<Post>> {
        Ok(self
            .0
            .begin_read()?
            .open_table(POST)?
            .get(id)?
            .map(|p| Post {
                id,
                message: p.value(),
            }))
    }

    pub fn posts(
        &self,
        keyword: Option<&str>,
        sort_order: Option<(Sort, Order)>,
        start: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Posts> {
        let keys = self._posts(keyword, sort_order)?;
        let total = keys.len();
        let l = limit.unwrap_or(total);

        let mut posts = Vec::with_capacity(total);
        for id in keys.into_iter().skip(start.unwrap_or_default()).take(l) {
            posts.push(self.post(id)?.unwrap())
        }

        Ok(Posts { total, posts })
    }

    fn _posts(&self, keyword: Option<&str>, sort_order: Option<(Sort, Order)>) -> Result<Vec<i64>> {
        let mut posts: Vec<i64> = self
            .0
            .begin_read()?
            .open_table(POST)?
            .iter()?
            .filter_map(|p| {
                let p = p.ok()?;
                if let Some(k) = keyword
                    && !k.trim_matches(S).is_empty()
                    && !p.1.value().to_lowercase().contains(&k.to_lowercase())
                {
                    None
                } else {
                    Some(p.0.value())
                }
            })
            .collect();
        if let Some((sort, order)) = sort_order {
            match sort {
                Sort::Time => match order {
                    //Order::Asc => posts.sort_by(|a, b| a.cmp(b)),
                    Order::Desc => posts.sort_by(|a, b| b.cmp(a)),
                },
            }
        }
        Ok(posts)
    }
}

/// Search keyword separators
const S: &[char] = &[
    '_', '-', ':', ';', ',', '(', ')', '[', ']', '/', '!', '?', ' ', // @TODO make optional
];

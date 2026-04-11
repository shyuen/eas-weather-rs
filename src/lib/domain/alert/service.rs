use crate::domain::database::port::DatabaseRepo;

#[derive(Debug, Clone)]
pub struct AlertService<DR>
where
    DR: DatabaseRepo,
{
    db_repo: DR,
}

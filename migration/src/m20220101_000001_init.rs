use sea_orm_migration::prelude::*;

use crate::helper::current_timestamp_utc;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User::Nickname)
                            .string_len(30)
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::Email)
                            .string_len(320)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(User::PasswordHash).string().not_null())
                    .col(
                        ColumnDef::new(User::IsAdmin)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(User::IsModerator)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(User::IsFaded)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(current_timestamp_utc()),
                    )
                    .col(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(current_timestamp_utc()),
                    )
                    .col(ColumnDef::new(User::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SilencedUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SilencedUser::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SilencedUser::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(SilencedUser::Reason)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(SilencedUser::EndDate).date().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-silenced_user-user_id")
                            .from(SilencedUser::Table, SilencedUser::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Title::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Title::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Title::Name)
                            .string_len(75)
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Title::IsVisible)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Entry::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Entry::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Entry::UserId).integer().not_null())
                    .col(ColumnDef::new(Entry::TitleId).integer().not_null())
                    .col(
                        ColumnDef::new(Entry::Content)
                            .string_len(65535)
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Entry::NetVotes)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Entry::TotalFavorites)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Entry::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(current_timestamp_utc()),
                    )
                    .col(
                        ColumnDef::new(Entry::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(current_timestamp_utc()),
                    )
                    .col(ColumnDef::new(Entry::DeletedAt).timestamp().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-entry-user_id")
                            .from(Entry::Table, Entry::UserId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-entry-title_id")
                            .from(Entry::Table, Entry::TitleId)
                            .to(Title::Table, Title::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Upvote::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Upvote::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Upvote::EntryId).integer().not_null())
                    .col(ColumnDef::new(Upvote::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(Upvote::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(current_timestamp_utc()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-upvote-entry_id")
                            .from(Upvote::Table, Upvote::EntryId)
                            .to(Entry::Table, Entry::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-upvote-user_id")
                            .from(Upvote::Table, Upvote::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Downvote::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Downvote::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Downvote::EntryId).integer().not_null())
                    .col(ColumnDef::new(Downvote::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(Downvote::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(current_timestamp_utc()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-downvote-entry_id")
                            .from(Downvote::Table, Downvote::EntryId)
                            .to(Entry::Table, Entry::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-downvote-user_id")
                            .from(Downvote::Table, Downvote::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Favorite::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Favorite::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Favorite::EntryId).integer().not_null())
                    .col(ColumnDef::new(Favorite::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(Favorite::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(current_timestamp_utc()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-favorite-entry_id")
                            .from(Favorite::Table, Favorite::EntryId)
                            .to(Entry::Table, Entry::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-favorite-user_id")
                            .from(Favorite::Table, Favorite::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Token::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Token::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Token::UserId).integer().not_null())
                    .col(ColumnDef::new(Token::Hash).string().not_null())
                    .col(
                        ColumnDef::new(Token::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(current_timestamp_utc()),
                    )
                    .col(ColumnDef::new(Token::InvalidatedAt).timestamp().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-token-user_id")
                            .from(Token::Table, Token::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Favorite::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Downvote::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Upvote::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Entry::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Title::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Token::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(SilencedUser::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Nickname,
    Email,
    PasswordHash,
    IsAdmin,
    IsModerator,
    IsFaded,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum SilencedUser {
    Table,
    Id,
    UserId,
    Reason,
    EndDate,
}

#[derive(DeriveIden)]
enum Title {
    Table,
    Id,
    Name,
    IsVisible,
}

#[derive(DeriveIden)]
enum Entry {
    Table,
    Id,
    UserId,
    TitleId,
    Content,
    NetVotes,
    TotalFavorites,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveIden)]
enum Upvote {
    Table,
    Id,
    EntryId,
    UserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Downvote {
    Table,
    Id,
    EntryId,
    UserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Favorite {
    Table,
    Id,
    EntryId,
    UserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Token {
    Table,
    Id,
    UserId,
    Hash,
    CreatedAt,
    InvalidatedAt,
}

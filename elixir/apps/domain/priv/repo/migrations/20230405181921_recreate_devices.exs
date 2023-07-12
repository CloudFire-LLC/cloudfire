defmodule Domain.Repo.Migrations.RecreateDevices do
  use Ecto.Migration

  def change do
    drop(table(:devices))

    create table(:devices, primary_key: false) do
      add(:id, :uuid, primary_key: true)
      add(:external_id, :string, null: false)

      add(:name, :string, null: false)

      add(:public_key, :string, null: false)

      add(
        :ipv4,
        references(:network_addresses,
          column: :address,
          type: :inet,
          with: [account_id: :account_id]
        )
      )

      add(
        :ipv6,
        references(:network_addresses,
          column: :address,
          type: :inet,
          with: [account_id: :account_id]
        )
      )

      add(:last_seen_user_agent, :string, null: false)
      add(:last_seen_remote_ip, :inet, null: false)
      add(:last_seen_version, :string, null: false)
      add(:last_seen_at, :utc_datetime_usec, null: false)

      add(:account_id, references(:accounts, type: :binary_id), null: false)
      add(:user_id, references(:users, type: :binary_id), null: false)

      add(:deleted_at, :utc_datetime_usec)
      timestamps(type: :utc_datetime_usec)
    end

    # Used to list devices for a user
    create(index(:devices, [:user_id], where: "deleted_at IS NULL"))

    # Used for upserts
    create(
      index(:devices, [:account_id, :user_id, :external_id],
        unique: true,
        where: "deleted_at IS NULL"
      )
    )

    # Used to enforce unique IPv4 and IPv6 addresses.
    create(index(:devices, [:account_id, :ipv4], unique: true, where: "deleted_at IS NULL"))
    create(index(:devices, [:account_id, :ipv6], unique: true, where: "deleted_at IS NULL"))

    # Used to enforce unique names and public keys.
    create(
      index(:devices, [:account_id, :user_id, :name], unique: true, where: "deleted_at IS NULL")
    )

    create(
      index(:devices, [:account_id, :user_id, :public_key],
        unique: true,
        where: "deleted_at IS NULL"
      )
    )
  end
end

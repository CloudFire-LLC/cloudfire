defmodule Web.RelayGroups.NewToken do
  use Web, :live_view
  alias Domain.Relays

  def mount(%{"id" => id}, _session, socket) do
    with true <- Domain.Config.self_hosted_relays_enabled?(),
         {:ok, group} <- Relays.fetch_group_by_id(id, socket.assigns.subject) do
      {group, env} =
        if connected?(socket) do
          {:ok, group} =
            Relays.update_group(%{group | tokens: []}, %{tokens: [%{}]}, socket.assigns.subject)

          :ok = Relays.subscribe_for_relays_presence_in_group(group)

          token = Relays.encode_token!(hd(group.tokens))
          {group, env(token)}
        else
          {group, nil}
        end

      {:ok,
       assign(socket,
         group: group,
         env: env,
         connected?: false,
         selected_tab: "systemd-instructions"
       )}
    else
      _other -> raise Web.LiveErrors.NotFoundError
    end
  end

  def render(assigns) do
    ~H"""
    <.breadcrumbs account={@account}>
      <.breadcrumb path={~p"/#{@account}/relay_groups"}>Relays</.breadcrumb>
      <.breadcrumb path={~p"/#{@account}/relay_groups/#{@group}"}>
        <%= @group.name %>
      </.breadcrumb>
      <.breadcrumb path={~p"/#{@account}/relay_groups/#{@group}/new_token"}>Deploy</.breadcrumb>
    </.breadcrumbs>

    <.section>
      <:title>
        Deploy a new Relay
      </:title>
      <:content>
        <div class="py-8 px-4 mx-auto max-w-2xl lg:py-16">
          <div class="text-xl mb-2">
            Select deployment method:
          </div>

          <.tabs :if={@env} id="deployment-instructions">
            <:tab
              id="systemd-instructions"
              label="systemd"
              phx_click="tab_selected"
              selected={@selected_tab == "systemd-instructions"}
            >
              <p class="p-4">
                1. Create an unprivileged user and group to run the relay:
              </p>

              <.code_block
                id="code-sample-systemd0"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
              >sudo groupadd -f firezone \
    && id -u firezone &>/dev/null || sudo useradd -r -g firezone -s /sbin/nologin firezone</.code_block>

              <p class="p-4">
                2. Create a new systemd unit file:
              </p>

              <.code_block
                id="code-sample-systemd1"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
              >sudo nano /etc/systemd/system/firezone-relay.service</.code_block>

              <p class="p-4">
                3. Copy-paste the following contents into the file:
              </p>

              <.code_block
                id="code-sample-systemd2"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
                phx-update="ignore"
              ><%= systemd_command(@env) %></.code_block>

              <p class="p-4">
                4. Save by pressing <kbd>Ctrl</kbd>+<kbd>X</kbd>, then <kbd>Y</kbd>, then <kbd>Enter</kbd>.
              </p>

              <p class="p-4">
                5. Reload systemd configuration:
              </p>

              <.code_block
                id="code-sample-systemd4"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
              >sudo systemctl daemon-reload</.code_block>

              <p class="p-4">
                6. Start the service:
              </p>

              <.code_block
                id="code-sample-systemd5"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
              >sudo systemctl start firezone-relay</.code_block>

              <p class="p-4">
                7. Enable the service to start on boot:
              </p>

              <.code_block
                id="code-sample-systemd6"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
              >sudo systemctl enable firezone-relay</.code_block>
              <hr />

              <h4 class="p-4 text-xl font-semibold">
                Troubleshooting
              </h4>

              <p class="p-4">
                Check the status of the service:
              </p>

              <.code_block
                id="code-sample-systemd7"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
              >sudo systemctl status firezone-relay</.code_block>

              <p class="p-4">
                Check the logs:
              </p>

              <.code_block
                id="code-sample-systemd8"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
              >sudo journalctl -u firezone-relay.service</.code_block>
            </:tab>
            <:tab
              id="docker-instructions"
              label="Docker"
              phx_click="tab_selected"
              selected={@selected_tab == "docker-instructions"}
            >
              <p class="p-4">
                Copy-paste this command to your server and replace <code>PUBLIC_IP4_ADDR</code>
                and <code>PUBLIC_IP6_ADDR</code>
                with your public IP addresses:
              </p>

              <.code_block
                id="code-sample-docker1"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
                phx-update="ignore"
              ><%= docker_command(@env) %></.code_block>

              <hr />

              <h4 class="p-4 text-xl font-semibold">
                Troubleshooting
              </h4>

              <p class="p-4">
                Check the container status:
              </p>

              <.code_block
                id="code-sample-docker2"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
              >docker ps --filter "name=firezone-relay"</.code_block>

              <p class="p-4">
                Check the container logs:
              </p>

              <.code_block
                id="code-sample-docker3"
                class="w-full text-xs whitespace-pre-line"
                phx-no-format
              >docker logs firezone-relay</.code_block>
            </:tab>
          </.tabs>

          <div id="connection-status" class="flex justify-between items-center">
            <p class="text-sm">
              Relay not connecting? See our <.link
                class="text-accent-500 hover:underline"
                href="https://www.firezone.dev/kb/administer/troubleshooting#relay-not-connecting"
              >relay troubleshooting guide</.link>.
            </p>
            <.initial_connection_status
              :if={@env}
              type="relay"
              navigate={~p"/#{@account}/relay_groups/#{@group}"}
              connected?={@connected?}
            />
          </div>
        </div>
      </:content>
    </.section>
    """
  end

  defp major_minor_version do
    vsn =
      Application.spec(:domain)
      |> Keyword.fetch!(:vsn)
      |> List.to_string()
      |> Version.parse!()

    "#{vsn.major}.#{vsn.minor}"
  end

  defp env(token) do
    api_url_override =
      if api_url = Domain.Config.get_env(:web, :api_url_override) do
        {"FIREZONE_API_URL", api_url}
      end

    [
      {"FIREZONE_ID", Ecto.UUID.generate()},
      {"FIREZONE_TOKEN", token},
      {"PUBLIC_IP4_ADDR", "YOU_MUST_SET_THIS_VALUE"},
      {"PUBLIC_IP6_ADDR", "YOU_MUST_SET_THIS_VALUE"},
      api_url_override,
      {"RUST_LOG",
       Enum.join(
         [
           "firezone_relay=trace",
           "firezone_tunnel=trace",
           "connlib_shared=trace",
           "tunnel_state=trace",
           "phoenix_channel=debug",
           "warn"
         ],
         ","
       )},
      {"LOG_FORMAT", "google-cloud"}
    ]
    |> Enum.reject(&is_nil/1)
  end

  defp docker_command(env) do
    [
      "docker run -d",
      "--restart=unless-stopped",
      "--pull=always",
      "--health-cmd=\"lsof -i UDP | grep firezone-relay\"",
      "--name=firezone-relay",
      "--cap-add=NET_ADMIN",
      "--volume /var/lib/firezone",
      "--sysctl net.ipv4.ip_forward=1",
      "--sysctl net.ipv4.conf.all.src_valid_mark=1",
      "--sysctl net.ipv6.conf.all.disable_ipv6=0",
      "--sysctl net.ipv6.conf.all.forwarding=1",
      "--sysctl net.ipv6.conf.default.forwarding=1",
      "--device=\"/dev/net/tun:/dev/net/tun\"",
      Enum.map(env, fn {key, value} -> "--env #{key}=\"#{value}\"" end),
      "--env FIREZONE_NAME=$(hostname)",
      "#{Domain.Config.fetch_env!(:domain, :docker_registry)}/relay:#{major_minor_version()}"
    ]
    |> List.flatten()
    |> Enum.join(" \\\n  ")
  end

  defp systemd_command(env) do
    """
    [Unit]
    Description=Firezone Relay
    After=network.target
    Documentation=https://www.firezone.dev/kb

    [Service]
    Type=simple
    #{Enum.map_join(env, "\n", fn {key, value} -> "Environment=\"#{key}=#{value}\"" end)}
    ExecStartPre=/bin/sh -c 'set -ue; \\
      if [ ! -e /usr/local/bin/firezone-relay ]; then \\
        FIREZONE_VERSION=$(curl -Ls \\
          -H "Accept: application/vnd.github+json" \\
          -H "X-GitHub-Api-Version: 2022-11-28" \\
          "https://api.github.com/repos/firezone/firezone/releases/latest" | \\
          grep "\\\\"tag_name\\\\":" | sed "s/.*\\\\"tag_name\\\\": \\\\"\\([^\\\\"\\\\]*\\).*/\\1/" \\
        ); \\
        [ "$FIREZONE_VERSION" = "" ] && echo "[Error] Can not fetch latest version, rate limited by GitHub?" && exit 1; \\
        echo "Downloading Firezone Relay version $FIREZONE_VERSION"; \\
        arch=$(uname -m); \\
        case $arch in \\
          aarch64) \\
            bin_url="https://github.com/firezone/firezone/releases/download/$FIREZONE_VERSION/relay-arm64" ;; \\
          armv7l) \\
            bin_url="https://github.com/firezone/firezone/releases/download/$FIREZONE_VERSION/relay-arm" ;; \\
          x86_64) \\
            bin_url="https://github.com/firezone/firezone/releases/download/$FIREZONE_VERSION/relay-x64" ;; \\
          *) \\
            echo "Unsupported architecture"; \\
            exit 1 ;; \\
        esac; \\
        curl -Ls $bin_url -o /usr/local/bin/firezone-relay; \\
        chgrp firezone /usr/local/bin/firezone-relay; \\
        chmod 0750 /usr/local/bin/firezone-relay; \\
      fi; \\
      mkdir -p /var/lib/firezone; \\
      chown firezone:firezone /var/lib/firezone; \\
      chmod 0775 /var/lib/firezone; \\
    '
    ExecStart=/usr/bin/sudo \\
      --preserve-env=FIREZONE_NAME,FIREZONE_ID,FIREZONE_TOKEN,PUBLIC_IP4_ADDR,PUBLIC_IP6_ADDR,RUST_LOG,LOG_FORMAT \\
      -u firezone \\
      -g firezone \\
      /usr/local/bin/firezone-relay
    TimeoutStartSec=3s
    TimeoutStopSec=15s
    Restart=always
    RestartSec=7

    [Install]
    WantedBy=multi-user.target
    """
  end

  def handle_event("tab_selected", %{"id" => id}, socket) do
    {:noreply, assign(socket, selected_tab: id)}
  end

  def handle_info(%Phoenix.Socket.Broadcast{topic: "relay_groups:" <> _group_id}, socket) do
    {:noreply, assign(socket, connected?: true)}
  end
end

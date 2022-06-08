defmodule FzHttpWeb.SidebarComponentTest do
  use FzHttpWeb.ConnCase, async: true

  alias FzHttpWeb.SidebarComponent

  describe "nav_class/2" do
    test "it computes nav class for account route" do
      assert SidebarComponent.nav_class("/account/something", "/account") ==
               "is-active has-icon"
    end

    test "it defaults to has-icon" do
      assert SidebarComponent.nav_class("/bar", "/foo") == "has-icon"
    end
  end
end

//
//  AppView.swift
//  (c) 2023 Firezone, Inc.
//  LICENSE: Apache-2.0
//

import _SwiftUINavigationState
import Combine
import Dependencies
import SwiftUI
import SwiftUINavigation

@MainActor
public final class AppViewModel: ObservableObject {
  @Published var welcomeViewModel: WelcomeViewModel?

  public init() {
    Task {
      let tunnel = try await TunnelStore.loadOrCreate()
      self.welcomeViewModel = WelcomeViewModel(
        appStore: AppStore(
          tunnelStore: TunnelStore(
            tunnel: tunnel
          )
        )
      )
    }
  }
}

public struct AppView: View {
  @ObservedObject var model: AppViewModel

  public init(model: AppViewModel) {
    self.model = model
  }

  @ViewBuilder
  public var body: some View {
    if let model = model.welcomeViewModel {
      WelcomeView(model: model)
    } else {
      ProgressView()
    }
  }
}

struct AppView_Previews: PreviewProvider {
  static var previews: some View {
    AppView(model: AppViewModel())
  }
}

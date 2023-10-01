import React from 'react';
import { ROOT_ID } from './utils/constants'
import { render, screen, waitFor } from '@testing-library/react';
import Speaker from './pages/Speakers'
import { SpeakerProvider } from './state/speaker';
import { FilterProvider } from './state/filter'
import { VersionProvider } from './state/version'
import App, { deriveAppliedVersion } from './App';
import { createMemoryRouter, RouterProvider } from "react-router-dom";

describe("UI", () => {
  test('renders configuration version', async () => {
    const router = createMemoryRouter([
      {

        path: "/",
        element: <App />,
        id: ROOT_ID,
        errorElement: <p>Uh oh, 404</p>,
        loader: () => ({ versions: [{ version: 1, appliedVersion: true, versionDate: "2023" }], speakers: undefined, filters: undefined, appliedVersion: undefined }),
        children: [{ path: "/", element: <div>Hello</div> }]
      },

    ], { initialEntries: ["/"] });
    render(<SpeakerProvider>
      <FilterProvider>
        <VersionProvider><RouterProvider router={router} /></VersionProvider>
      </FilterProvider>
    </SpeakerProvider>)
    await waitFor(() => expect(screen.getByText(/Hello/i)).toBeInTheDocument())

  });
  test('correct speaker configuration', async () => {
    const router = createMemoryRouter([
      {

        path: "/",
        element: <App />,
        id: ROOT_ID,
        errorElement: <p>Uh oh, 404</p>,
        loader: () => ({
          versions: [{ version: "0.1", appliedVersion: true }],
          speakers: [{
            speaker: "sp1",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
          },
          {
            speaker: "sp2",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
          },
          {
            speaker: "sp3",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
          },
          {
            speaker: "sp4",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
          },
          {
            speaker: "sp5",
            isSubwoofer: true,
            crossover: 100,
            delay: 4,
            gain: 2
          }],
          filters: undefined,
          appliedVersion: undefined
        }),
        children: [{ path: "/speakers", element: <Speaker /> }]
      },

    ], { initialEntries: ["/speakers"] });
    render(<SpeakerProvider>
      <FilterProvider>
        <VersionProvider><RouterProvider router={router} /></VersionProvider>
      </FilterProvider>
    </SpeakerProvider>)
    const select = await waitFor(() => screen.getAllByTitle('4.1').at(0))
    expect(select?.textContent).toEqual("4.1")
  });
})

describe("deriveAppliedVersion", () => {
  it("gets applied version if exists", () => {
    expect(deriveAppliedVersion([{ version: 2, appliedVersion: true, versionDate: "2023" }])).toEqual(2)
  })
  it("gets applied version if exists with more versions", () => {
    expect(deriveAppliedVersion([{ version: 1, appliedVersion: true, versionDate: "2023" }, { version: 2, appliedVersion: false, versionDate: "2023" }])).toEqual(1)
  })
  it("returns last element if nothing applied", () => {
    expect(deriveAppliedVersion([{ version: 1, appliedVersion: false, versionDate: "2023" }, { version: 2, appliedVersion: false, versionDate: "2023" }])).toEqual(2)
  })
})
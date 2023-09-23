import React from 'react';
import { ROOT_ID } from './utils/constants'
import { render, screen, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event'

import { SpeakerProviderComponent } from './state/speaker';
import { FilterProviderComponent } from './state/filter'
import { VersionProviderComponent } from './state/version'
import App, { deriveAppliedVersion } from './App';
import {
  createMemoryRouter,
  RouterProvider
} from "react-router-dom";

describe("UI", () => {
  test('renders configuration version', async () => {
    const router = createMemoryRouter([
      {

        path: "/",
        element: <App />,
        id: ROOT_ID,
        errorElement: <p>Uh oh, 404</p>,
        loader: () => ({ versions: [{ version: "0.1", appliedVersion: true }], speakers: undefined, filters: undefined, appliedVersion: undefined }),
        children: [{ path: "/", element: <div>Hello</div> }]
      },

    ], { initialEntries: ["/"] });
    render(<SpeakerProviderComponent>
      <FilterProviderComponent>
        <VersionProviderComponent><RouterProvider router={router} /></VersionProviderComponent>
      </FilterProviderComponent>
    </SpeakerProviderComponent>)
    await waitFor(() => expect(screen.getByText(/Select Configuration Version/i)).toBeInTheDocument())

  });

  test('correct version displays and update made', async () => {
    const spy = jest.fn((_: string) => Promise.resolve({ speakers: [], filters: [] }))
    const router = createMemoryRouter([
      {

        path: "/",
        element: <App getConfigurationProp={spy} />,
        id: ROOT_ID,
        errorElement: <p>Uh oh, 404</p>,
        loader: () => ({ versions: [{ version: "0.1", appliedVersion: true }, { version: "0.2", appliedVersion: true }], speakers: undefined, filters: undefined, appliedVersion: undefined }),
        children: [{ path: "/", element: <div>Hello</div> }]
      },

    ], { initialEntries: ["/"] });

    render(<SpeakerProviderComponent>
      <FilterProviderComponent>
        <VersionProviderComponent><RouterProvider router={router} /></VersionProviderComponent>
      </FilterProviderComponent>
    </SpeakerProviderComponent>)

    const select = await waitFor(() => screen.getByRole('combobox'))
    await act(async () => await userEvent.click(select))
    const initSelect = await waitFor(() => screen.getByTitle('0.1'))
    expect(initSelect.className).toContain("ant-select-item-option-active")

    await waitFor(() => screen.getByTitle('0.2'))
    await act(async () => await userEvent.click(screen.getByTitle('0.2')))
    const secondSelect = await waitFor(() => screen.getAllByTitle('0.2').at(1))
    expect(secondSelect?.className).toContain("ant-select-item-option-active")

    expect(spy).toHaveBeenCalledWith("0.2")
  });

})

describe("deriveAppliedVersion", () => {
  it("gets applied version if exists", () => {
    expect(deriveAppliedVersion([{ version: "0.2", appliedVersion: true }])).toEqual("0.2")
  })
  it("gets applied version if exists with more versions", () => {
    expect(deriveAppliedVersion([{ version: "0.1", appliedVersion: true }, { version: "0.2", appliedVersion: false }])).toEqual("0.1")
  })
  it("returns last element if nothing applied", () => {
    expect(deriveAppliedVersion([{ version: "0.1", appliedVersion: false }, { version: "0.2", appliedVersion: false }])).toEqual("0.2")
  })
})
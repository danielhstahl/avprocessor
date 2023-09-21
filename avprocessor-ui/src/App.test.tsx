import React from 'react';
import { render, screen } from '@testing-library/react';
import App from './App';
import {
  createMemoryRouter,
  RouterProvider,
} from "react-router-dom";
test('renders learn react link', () => {
  const router = createMemoryRouter([
    {
      path: "/",
      element: <App />,
      id: ".",
      errorElement: <p>Uh oh, 404</p>,
    },

  ]);
  render(<RouterProvider router={router} />);
  const configVersion = screen.getByText(/Select Configuration Version/i);
  expect(configVersion).toBeInTheDocument();
});

import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App, { loader, MenuItems } from './App';
import reportWebVitals from './reportWebVitals';
import {
  createBrowserRouter,
  RouterProvider,
} from "react-router-dom";
import { ROOT_ID } from './utils/constants'
import { SpeakerProviderComponent } from './state/speaker';
import { FilterProviderComponent } from './state/filter'
import { VersionProviderComponent } from './state/version'
const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);
const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    id: ROOT_ID,
    errorElement: <p>Uh oh, 404</p>,
    loader,
    children: MenuItems.map(({ key, element }) => ({ path: key, element }))
  },

]);
root.render(
  <React.StrictMode>
    <SpeakerProviderComponent>
      <FilterProviderComponent>
        <VersionProviderComponent>
          <RouterProvider router={router} />
        </VersionProviderComponent>
      </FilterProviderComponent>
    </SpeakerProviderComponent>
  </React.StrictMode>
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
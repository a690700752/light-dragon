import { BankOutlined, ProfileOutlined } from "@ant-design/icons";
import { Layout, Menu } from "antd";
import React from "react";
import { RepoManage } from "./RepoManage";

const { Content, Sider } = Layout;

const App: React.FC = () => {
  return (
    <Layout>
      <Sider
        breakpoint="lg"
        collapsedWidth="0"
        onBreakpoint={(broken) => {
          console.log(broken);
        }}
        onCollapse={(collapsed, type) => {
          console.log(collapsed, type);
        }}
      >
        <div className="demo-logo-vertical" />
        <Menu
          theme="dark"
          mode="inline"
          defaultSelectedKeys={["repo"]}
          items={[
            {
              key: "repo",
              label: "仓库管理",
              icon: <BankOutlined />,
            },
            {
              key: "env",
              label: "环境变量",
              icon: <ProfileOutlined />,
            },
          ]}
        />
      </Sider>
      <Layout>
        {/* <Header style={{ padding: 0, background: colorBgContainer }} /> */}
        <Content style={{ margin: "24px 16px 0" }}>
          <RepoManage />
        </Content>
      </Layout>
    </Layout>
  );
};

export default App;

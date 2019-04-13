# [microk8s](https://microk8s.io/)
Zero-ops k8s on just about any Linux box.

## Add ons
To get started install the following add ons:
1. dns 
2. dashboard
3. registry (also installs storage)

## FAQ

### How to connect to the k8s dashboard?
1. Run the following command to discover the dasboard IP Address
    ```
    microk8s.kubectl get all --all-namespaces |grep kubernetes-dashboard | grep ClusterIP
    ```    
2. When asked to authenticate, click on \[Skip\]
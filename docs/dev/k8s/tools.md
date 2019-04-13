## [kustomize](https://github.com/kubernetes-sigs/kustomize)
kustomize lets you customize raw, template-free YAML files for multiple purposes, leaving the original YAML untouched and usable as is.

**Note** Kustomize is now available on kubectl v1.14 and can be used by specifying a directory containing a kustomization.yaml: `kubectl apply -k dir/`

## [gitkube](https://gitkube.sh/)
Gitkube is a tool for building and deploying docker images on Kubernetes using git push.

## [prometheus operator](https://github.com/coreos/prometheus-operator)
[kube-prometheus](https://github.com/coreos/prometheus-operator/tree/master/contrib/kube-prometheus) sudo

## [Diffing Local and Cluster State](https://kubectl.docs.kubernetes.io/pages/app_composition_and_deployment/diffing_local_and_remote_resources.html)

```
export KUBECTL_EXTERNAL_DIFF=meld

``` 
- http://meldmerge.org/

## Editing resources 
According to the editor can be set via:
```
export KUBE_EDITOR='code --wait'
```
However, this is not working - at least with microk8s.kubectl

## [SKAFFOLD](https://github.com/GoogleContainerTools/skaffold)
Skaffold is a command line tool that facilitates continuous development for Kubernetes applications. You can iterate on 
your application source code locally then deploy to local or remote Kubernetes clusters. Skaffold handles the workflow 
for building, pushing and deploying your application. It also provides building blocks and describe customizations for 
a CI/CD pipeline.